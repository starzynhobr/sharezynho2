use serde::{Deserialize, Serialize};
use std::sync::atomic::Ordering;
use std::sync::{Arc, Mutex};
use tauri::{AppHandle, Emitter, State};

use crate::client::{ClientEvent, ClientSession};
use crate::network::tailscale::{detect_lan_ipv4, detect_tailscale_ipv4};
use crate::protocol::{DEFAULT_TCP_PORT, DEFAULT_UDP_PORT, ParticipantInfo};
use crate::screen::ScreenCapture;
use crate::server::RoomServer;
use crate::stream_server::DEFAULT_STREAM_PORT;

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

pub struct AppState {
    pub session: Arc<tokio::sync::Mutex<Option<ClientSession>>>,
    pub server_shutdown: Arc<tokio::sync::Mutex<Option<tokio::sync::watch::Sender<bool>>>>,
    pub latest_frame: Arc<Mutex<Option<Vec<u8>>>>,
    pub preferred_ip: String,
    pub is_tailscale: bool,
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}

impl AppState {
    pub fn new() -> Self {
        let tailscale_ip = detect_tailscale_ipv4();
        let is_tailscale = tailscale_ip.is_some();
        let preferred_ip = tailscale_ip
            .or_else(detect_lan_ipv4)
            .unwrap_or_else(|| "127.0.0.1".to_string());

        Self {
            session: Arc::new(tokio::sync::Mutex::new(None)),
            server_shutdown: Arc::new(tokio::sync::Mutex::new(None)),
            latest_frame: Arc::new(Mutex::new(None)),
            preferred_ip,
            is_tailscale,
        }
    }
}

#[tauri::command]
pub fn get_network_info(state: State<'_, AppState>) -> NetworkInfo {
    NetworkInfo {
        preferred_ip: state.preferred_ip.clone(),
        is_tailscale: state.is_tailscale,
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

    // Share frame pointer with session
    let session_latest_frame = session.latest_screen_frame.clone();
    let app_latest_frame = state.latest_frame.clone();

    // Link incoming frames to app stream
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(std::time::Duration::from_millis(30));
        loop {
            interval.tick().await;
            let frame = {
                let guard = session_latest_frame.lock().unwrap();
                guard.clone()
            };
            if let Some(f) = frame {
                let mut guard = app_latest_frame.lock().unwrap();
                *guard = Some(f);
            }
        }
    });

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
        host_address: format!("{}:{}", state.preferred_ip, DEFAULT_TCP_PORT),
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

    let session_latest_frame = session.latest_screen_frame.clone();
    let app_latest_frame = state.latest_frame.clone();

    tokio::spawn(async move {
        let mut interval = tokio::time::interval(std::time::Duration::from_millis(30));
        loop {
            interval.tick().await;
            let frame = {
                let guard = session_latest_frame.lock().unwrap();
                guard.clone()
            };
            if let Some(f) = frame {
                let mut guard = app_latest_frame.lock().unwrap();
                *guard = Some(f);
            }
        }
    });

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
                    let _ = app_handle.emit("disconnected", ());
                    break;
                }
                _ => {}
            }
        }
    });
}
