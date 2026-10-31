use serde::{Deserialize, Serialize};
use std::sync::atomic::Ordering;
use std::sync::{Arc, Mutex};
use tauri::{AppHandle, Emitter, Manager, State};

use crate::audio;
use crate::client::{ClientEvent, ClientSession};
use crate::network::tailscale::{detect_lan_ipv4, detect_tailscale_ipv4};
use crate::protocol::{DEFAULT_TCP_PORT, DEFAULT_UDP_PORT, ParticipantInfo};
use crate::screen::ScreenCapture;
use crate::server::RoomServer;
use crate::stream_server::DEFAULT_STREAM_PORT;

const RECONNECT_ATTEMPTS: u32 = 5;
const RECONNECT_DELAY: std::time::Duration = std::time::Duration::from_secs(2);

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct NetworkInfo {
    pub preferred_ip: String,
    pub is_tailscale: bool,
    pub tcp_port: u16,
    pub udp_port: u16,
    pub stream_port: u16,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct RoomJoinedInfo {
    pub room: String,
    pub display_name: String,
    pub is_host: bool,
    pub host_address: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct RoomStatePayload {
    pub room: String,
    pub participants: Vec<ParticipantInfo>,
    pub screen_presenter: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct AudioDevices {
    pub inputs: Vec<String>,
    pub outputs: Vec<String>,
}

pub struct AppState {
    pub session: Arc<tokio::sync::Mutex<Option<ClientSession>>>,
    pub server_shutdown: Arc<tokio::sync::Mutex<Option<tokio::sync::watch::Sender<bool>>>>,
    pub latest_frame: Arc<Mutex<Option<Vec<u8>>>>,
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}

impl AppState {
    pub fn new() -> Self {
        Self {
            session: Arc::new(tokio::sync::Mutex::new(None)),
            server_shutdown: Arc::new(tokio::sync::Mutex::new(None)),
            latest_frame: Arc::new(Mutex::new(None)),
        }
    }
}

/// Detected on demand, since Tailscale may connect after the app opens.
#[tauri::command(async)]
pub fn get_network_info() -> NetworkInfo {
    let tailscale_ip = detect_tailscale_ipv4();
    NetworkInfo {
        is_tailscale: tailscale_ip.is_some(),
        preferred_ip: tailscale_ip
            .or_else(detect_lan_ipv4)
            .unwrap_or_else(|| "127.0.0.1".to_string()),
        tcp_port: DEFAULT_TCP_PORT,
        udp_port: DEFAULT_UDP_PORT,
        stream_port: DEFAULT_STREAM_PORT,
    }
}

#[tauri::command]
pub async fn create_room(
    name: String,
    room: String,
    state: State<'_, AppState>,
    app_handle: AppHandle,
) -> Result<RoomJoinedInfo, String> {
    // 1. Leave previous session if any
    leave_room_internal(&state).await;

    // 2. Start RoomServer
    let (shutdown_tx, shutdown_rx) = tokio::sync::watch::channel(false);
    {
        let mut s_guard = state.server_shutdown.lock().await;
        *s_guard = Some(shutdown_tx);
    }

    tokio::spawn(async move {
        if let Err(e) = RoomServer::start(DEFAULT_TCP_PORT, DEFAULT_UDP_PORT, shutdown_rx).await {
            eprintln!("RoomServer error: {}", e);
        }
    });

    tokio::time::sleep(std::time::Duration::from_millis(120)).await;

    // 3. Connect as local client
    let server_addr = format!("127.0.0.1:{}", DEFAULT_TCP_PORT);
    let (event_tx, event_rx) = std::sync::mpsc::channel();

    let session = ClientSession::connect(&server_addr, &room, &name, true, event_tx)
        .await
        .map_err(|e| format!("Falha ao conectar na sala local: {}", e))?;

    spawn_frame_bridge(&session, state.latest_frame.clone());

    let display_name = session.display_name.clone();
    let admitted_room = session.room_name.clone();
    let is_host = session.is_host;

    {
        let mut sess_guard = state.session.lock().await;
        *sess_guard = Some(session);
    }

    // Spawn event listener
    spawn_event_listener(event_rx, app_handle);

    Ok(RoomJoinedInfo {
        room: admitted_room,
        display_name,
        is_host,
        host_address: format!("{}:{}", get_network_info().preferred_ip, DEFAULT_TCP_PORT),
    })
}

#[tauri::command]
pub async fn join_room(
    address: String,
    room: String,
    name: String,
    state: State<'_, AppState>,
    app_handle: AppHandle,
) -> Result<RoomJoinedInfo, String> {
    leave_room_internal(&state).await;

    let server_addr = if address.contains(':') {
        address.clone()
    } else {
        format!("{}:{}", address, DEFAULT_TCP_PORT)
    };

    let (event_tx, event_rx) = std::sync::mpsc::channel();

    let session = ClientSession::connect(&server_addr, &room, &name, false, event_tx)
        .await
        .map_err(|e| format!("Falha ao conectar no host {}: {}", server_addr, e))?;

    spawn_frame_bridge(&session, state.latest_frame.clone());

    let display_name = session.display_name.clone();
    let admitted_room = session.room_name.clone();
    let is_host = session.is_host;

    {
        let mut sess_guard = state.session.lock().await;
        *sess_guard = Some(session);
    }

    spawn_event_listener(event_rx, app_handle);

    Ok(RoomJoinedInfo {
        room: admitted_room,
        display_name,
        is_host,
        host_address: server_addr,
    })
}

#[tauri::command]
pub async fn leave_room(state: State<'_, AppState>, app_handle: AppHandle) -> Result<(), String> {
    leave_room_internal(&state).await;
    let _ = app_handle.emit("disconnected", ());
    Ok(())
}

async fn leave_room_internal(state: &AppState) {
    {
        let mut sess_guard = state.session.lock().await;
        if let Some(mut sess) = sess_guard.take() {
            sess.disconnect();
        }
    }
    {
        let mut serv_guard = state.server_shutdown.lock().await;
        if let Some(tx) = serv_guard.take() {
            let _ = tx.send(true);
        }
    }
    {
        let mut frame_guard = state.latest_frame.lock().unwrap();
        *frame_guard = None;
    }
}

#[tauri::command]
pub async fn toggle_mic(state: State<'_, AppState>) -> Result<bool, String> {
    let mut sess_guard = state.session.lock().await;
    if let Some(sess) = sess_guard.as_mut() {
        let current = sess.is_mic_enabled.load(Ordering::Relaxed);
        let next = !current;
        sess.set_microphone(next);
        Ok(next)
    } else {
        Err("Nenhuma sessão ativa".to_string())
    }
}

#[tauri::command]
pub async fn toggle_deafen(state: State<'_, AppState>) -> Result<bool, String> {
    let mut sess_guard = state.session.lock().await;
    if let Some(sess) = sess_guard.as_mut() {
        let current = sess.is_deafened.load(Ordering::Relaxed);
        let next = !current;
        sess.set_deafened(next);
        Ok(next)
    } else {
        Err("Nenhuma sessão ativa".to_string())
    }
}

#[tauri::command]
pub fn get_monitors() -> Vec<String> {
    ScreenCapture::enumerate_monitors()
}

#[tauri::command]
pub async fn start_screen_share(
    monitor_index: usize,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let mut sess_guard = state.session.lock().await;
    if let Some(sess) = sess_guard.as_mut() {
        sess.start_screen_share(monitor_index)
    } else {
        Err("Nenhuma sessão ativa".to_string())
    }
}

#[tauri::command]
pub async fn stop_screen_share(state: State<'_, AppState>) -> Result<(), String> {
    let mut sess_guard = state.session.lock().await;
    if let Some(sess) = sess_guard.as_mut() {
        sess.stop_screen_share();
        Ok(())
    } else {
        Err("Nenhuma sessão ativa".to_string())
    }
}

#[tauri::command]
pub fn get_audio_devices() -> AudioDevices {
    AudioDevices {
        inputs: audio::input_device_names(),
        outputs: audio::output_device_names(),
    }
}

#[tauri::command]
pub fn set_audio_devices(input: Option<String>, output: Option<String>) {
    audio::set_preferred_devices(input, output);
}

/// Handles an unexpected drop: guests retry, hosts have nothing to return to.
async fn handle_connection_lost(app_handle: AppHandle) {
    let state = app_handle.state::<AppState>();

    let target = {
        let mut sess_guard = state.session.lock().await;
        let Some(sess) = sess_guard.as_mut() else {
            return;
        };
        sess.disconnect();
        (!sess.is_host).then(|| {
            (
                sess.server_addr.clone(),
                sess.room_name.clone(),
                sess.display_name.clone(),
                sess.is_mic_enabled.load(Ordering::Relaxed),
                sess.is_deafened.load(Ordering::Relaxed),
            )
        })
    };

    if let Some((server_addr, room, name, mic_enabled, deafened)) = target {
        for attempt in 1..=RECONNECT_ATTEMPTS {
            let _ = app_handle.emit(
                "room-error",
                format!("Conexão perdida. Reconectando ({attempt}/{RECONNECT_ATTEMPTS})..."),
            );
            tokio::time::sleep(RECONNECT_DELAY).await;

            let mut sess_guard = state.session.lock().await;
            if sess_guard.is_none() {
                return;
            }
            let (event_tx, event_rx) = std::sync::mpsc::channel();
            if let Ok(mut session) =
                ClientSession::connect(&server_addr, &room, &name, false, event_tx).await
            {
                session.set_microphone(mic_enabled);
                session.set_deafened(deafened);
                spawn_frame_bridge(&session, state.latest_frame.clone());
                *sess_guard = Some(session);
                drop(sess_guard);
                spawn_event_listener(event_rx, app_handle.clone());
                let _ = app_handle.emit("reconnected", ());
                return;
            }
        }
    }

    leave_room_internal(&state).await;
    let _ = app_handle.emit("disconnected", ());
}

/// Links the session's incoming frames to the app stream until the session ends.
fn spawn_frame_bridge(session: &ClientSession, app_latest_frame: Arc<Mutex<Option<Vec<u8>>>>) {
    let session_latest_frame = session.latest_screen_frame.clone();
    let mut shutdown_rx = session.subscribe_shutdown();

    tokio::spawn(async move {
        let mut interval = tokio::time::interval(std::time::Duration::from_millis(30));
        loop {
            tokio::select! {
                _ = shutdown_rx.changed() => break,
                _ = interval.tick() => {
                    let frame = session_latest_frame.lock().unwrap().clone();
                    *app_latest_frame.lock().unwrap() = frame;
                }
            }
        }
    });
}

fn spawn_event_listener(event_rx: std::sync::mpsc::Receiver<ClientEvent>, app_handle: AppHandle) {
    std::thread::spawn(move || {
        while let Ok(event) = event_rx.recv() {
            match event {
                ClientEvent::RoomStateUpdated {
                    room,
                    participants,
                    screen_presenter,
                } => {
                    let payload = RoomStatePayload {
                        room,
                        participants,
                        screen_presenter,
                    };
                    let _ = app_handle.emit("room-state", payload);
                }
                ClientEvent::Error(err) => {
                    let _ = app_handle.emit("room-error", err);
                }
                ClientEvent::Disconnected => {
                    tauri::async_runtime::block_on(handle_connection_lost(app_handle.clone()));
                    break;
                }
                _ => {}
            }
        }
    });
}
