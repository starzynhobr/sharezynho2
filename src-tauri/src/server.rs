use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::{TcpListener, UdpSocket};
use tokio::sync::{Mutex, mpsc, watch};

use crate::protocol::{
    ClientMessage, ParticipantInfo, ServerMessage, TAG_AUDIO, TAG_VIDEO, serialize_server_msg,
};

struct Participant {
    id: String,
    name: String,
    is_speaking: bool,
    is_sharing_screen: bool,
    is_host: bool,
    tcp_tx: mpsc::Sender<String>,
    udp_addr: SocketAddr,
}

struct RoomStateInner {
    room_name: String,
    participants: HashMap<String, Participant>,
    screen_presenter: Option<String>,
}

pub struct RoomServer {
    state: Arc<Mutex<RoomStateInner>>,
    udp_socket: Arc<UdpSocket>,
}

impl RoomServer {
    pub async fn start(
        tcp_port: u16,
        udp_port: u16,
        mut shutdown_rx: watch::Receiver<bool>,
    ) -> Result<tokio::task::JoinHandle<()>, Box<dyn std::error::Error + Send + Sync>> {
        let tcp_listener = TcpListener::bind(format!("0.0.0.0:{}", tcp_port)).await?;
        let udp_socket = Arc::new(UdpSocket::bind(format!("0.0.0.0:{}", udp_port)).await?);

        let state = Arc::new(Mutex::new(RoomStateInner {
            room_name: String::new(),
            participants: HashMap::new(),
            screen_presenter: None,
        }));

        let server = Arc::new(Self {
            state: state.clone(),
            udp_socket: udp_socket.clone(),
        });

        // 1. Spawn UDP relay loop
        let udp_server = server.clone();
        let mut udp_shutdown_rx = shutdown_rx.clone();
        tokio::spawn(async move {
            let mut buf = vec![0u8; 65535];
            loop {
                tokio::select! {
                    _ = udp_shutdown_rx.changed() => {
                        if *udp_shutdown_rx.borrow() {
                            break;
                        }
                    }
                    res = udp_server.udp_socket.recv_from(&mut buf) => {
                        if let Ok((len, src_addr)) = res
                            && len > 0
                        {
                            udp_server.relay_udp(&buf[..len], src_addr).await;
                        }
                    }
                }
            }
        });

        // 2. Spawn TCP accept loop
        let handle = tokio::spawn(async move {
            loop {
                tokio::select! {
                    _ = shutdown_rx.changed() => {
                        if *shutdown_rx.borrow() {
                            break;
                        }
                    }
                    accept_res = tcp_listener.accept() => {
                        if let Ok((socket, peer_addr)) = accept_res {
                            let state_clone = server.state.clone();
                            let client_shutdown_rx = shutdown_rx.clone();
                            tokio::spawn(async move {
                                if let Err(e) = Self::handle_client(socket, peer_addr, state_clone, udp_port, client_shutdown_rx).await {
                                    eprintln!("SERVER handle_client ERROR: {:?}", e);
                                }
                            });
                        }
                    }
                }
            }
        });

        Ok(handle)
    }

    async fn relay_udp(&self, data: &[u8], src_addr: SocketAddr) {
        if data.is_empty() {
            return;
        }
        let tag = data[0];
        if tag != TAG_AUDIO && tag != TAG_VIDEO {
            return;
        }

        let addrs: Vec<SocketAddr> = {
            let guard = self.state.lock().await;
            guard
                .participants
                .values()
                .map(|p| p.udp_addr)
                .filter(|addr| *addr != src_addr)
                .collect()
        };

        for addr in addrs {
            let _ = self.udp_socket.send_to(data, addr).await;
        }
    }

    async fn handle_client(
        stream: tokio::net::TcpStream,
        peer_addr: SocketAddr,
        state: Arc<Mutex<RoomStateInner>>,
        relay_udp_port: u16,
        mut shutdown_rx: watch::Receiver<bool>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let (reader, mut writer) = stream.into_split();
        let mut lines = BufReader::new(reader).lines();

        // 1. Await Join message
        let first_line = match lines.next_line().await? {
            Some(l) => l,
            None => return Ok(()),
        };

        let join_msg: ClientMessage = serde_json::from_str(&first_line)?;
        let (name, room, udp_port) = match join_msg {
            ClientMessage::Join {
                name,
                room,
                udp_port,
            } => (name, room, udp_port),
            _ => return Ok(()),
        };

        let participant_id = format!(
            "{}-{}",
            name.to_lowercase().replace(' ', "-"),
            fastrand::u32(1000..9999)
        );
        let client_udp_addr = SocketAddr::new(peer_addr.ip(), udp_port);

        let (tcp_tx, mut tcp_rx) = mpsc::channel::<String>(32);

        // Determine if first participant in room is host
        let is_host = {
            let mut guard = state.lock().await;
            if guard.room_name.is_empty() {
                guard.room_name = room.clone();
            }
            guard.participants.is_empty()
        };

        // Register participant
        {
            let mut guard = state.lock().await;
            guard.participants.insert(
                participant_id.clone(),
                Participant {
                    id: participant_id.clone(),
                    name: name.clone(),
                    is_speaking: false,
                    is_sharing_screen: false,
                    is_host,
                    tcp_tx: tcp_tx.clone(),
                    udp_addr: client_udp_addr,
                },
            );
        }

        // Send Admitted message
        let admitted = ServerMessage::Admitted {
            participant_id: participant_id.clone(),
            room: room.clone(),
            udp_relay_port: relay_udp_port,
        };
        let _ = writer
            .write_all(serialize_server_msg(&admitted).as_bytes())
            .await;

        // Broadcast updated room state
        Self::broadcast_state(&state).await;

        // Writer task
        let writer_handle = tokio::spawn(async move {
            while let Some(msg) = tcp_rx.recv().await {
                if writer.write_all(msg.as_bytes()).await.is_err() {
                    break;
                }
            }
        });

        // Reader loop
        let p_id = participant_id.clone();
        let state_reader = state.clone();
        loop {
            let line = tokio::select! {
                // Closing the socket tells guests that the room has ended
                _ = shutdown_rx.changed() => break,
                res = lines.next_line() => match res {
                    Ok(Some(line)) => line,
                    _ => break,
                },
            };
            if let Ok(msg) = serde_json::from_str::<ClientMessage>(&line) {
                match msg {
                    ClientMessage::Speaking { is_speaking } => {
                        let mut guard = state_reader.lock().await;
                        if let Some(p) = guard.participants.get_mut(&p_id) {
                            p.is_speaking = is_speaking;
                        }
                        drop(guard);
                        Self::broadcast_state(&state_reader).await;
                    }
                    ClientMessage::ScreenShareStarted { .. } => {
                        let mut guard = state_reader.lock().await;
                        guard.screen_presenter = Some(p_id.clone());
                        if let Some(p) = guard.participants.get_mut(&p_id) {
                            p.is_sharing_screen = true;
                        }
                        drop(guard);
                        Self::broadcast_state(&state_reader).await;
                    }
                    ClientMessage::ScreenShareStopped => {
                        let mut guard = state_reader.lock().await;
                        if guard.screen_presenter.as_deref() == Some(&p_id) {
                            guard.screen_presenter = None;
                        }
                        if let Some(p) = guard.participants.get_mut(&p_id) {
                            p.is_sharing_screen = false;
                        }
                        drop(guard);
                        Self::broadcast_state(&state_reader).await;
                    }
                    ClientMessage::Leave => {
                        break;
                    }
                    ClientMessage::Ping => {
                        let pong = serialize_server_msg(&ServerMessage::Pong);
                        let _ = tcp_tx.send(pong).await;
                    }
                    _ => {}
                }
            }
        }

        // Cleanup on disconnect
        {
            let mut guard = state.lock().await;
            guard.participants.remove(&participant_id);
            if guard.screen_presenter.as_deref() == Some(&participant_id) {
                guard.screen_presenter = None;
            }
        }
        Self::broadcast_state(&state).await;
        writer_handle.abort();

        Ok(())
    }

    async fn broadcast_state(state: &Arc<Mutex<RoomStateInner>>) {
        let guard = state.lock().await;
        let participants_info: Vec<ParticipantInfo> = guard
            .participants
            .values()
            .map(|p| ParticipantInfo {
                id: p.id.clone(),
                name: p.name.clone(),
                is_speaking: p.is_speaking,
                is_host: p.is_host,
                is_sharing_screen: p.is_sharing_screen,
            })
            .collect();

        let state_msg = ServerMessage::RoomState {
            room: guard.room_name.clone(),
            participants: participants_info,
            screen_presenter: guard.screen_presenter.clone(),
        };
        let serialized = serialize_server_msg(&state_msg);

        for p in guard.participants.values() {
            let _ = p.tcp_tx.try_send(serialized.clone());
        }
    }
}
