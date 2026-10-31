use std::net::SocketAddr;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::{TcpStream, UdpSocket};
use tokio::sync::{broadcast, mpsc, watch};

use crate::audio::processing::VoiceProcessor;
use crate::audio::{AudioCapture, AudioPlayback};
use crate::protocol::{
    ClientMessage, PEER_TIMEOUT_SECS, PING_INTERVAL_SECS, ParticipantInfo, ServerMessage,
    TAG_AUDIO, TAG_VIDEO, serialize_client_msg,
};
use crate::screen::{ScreenCapture, ScreenReceiver, VideoQuality};

const CONNECT_TIMEOUT: Duration = Duration::from_secs(5);

pub enum ClientEvent {
    SessionConnected(Box<ClientSession>),
    RoomStateUpdated {
        room: String,
        participants: Vec<ParticipantInfo>,
        screen_presenter: Option<String>,
    },
    Error(String),
    Disconnected,
}

pub struct ClientSession {
    #[allow(dead_code)]
    pub participant_id: String,
    pub room_name: String,
    pub display_name: String,
    pub server_addr: String,
    pub is_host: bool,
    tcp_tx: mpsc::Sender<String>,
    audio_capture: Option<AudioCapture>,
    audio_playback: Option<AudioPlayback>,
    screen_capture: Option<ScreenCapture>,
    screen_packet_tx: mpsc::Sender<Vec<u8>>,
    pub is_mic_enabled: Arc<AtomicBool>,
    pub is_deafened: Arc<AtomicBool>,
    pub is_sharing_screen: Arc<AtomicBool>,
    pub video_frame_tx: broadcast::Sender<Vec<u8>>,
    shutdown_tx: watch::Sender<bool>,
}

impl ClientSession {
    pub async fn connect(
        server_addr: &str,
        room: &str,
        name: &str,
        is_host: bool,
        event_tx: std::sync::mpsc::Sender<ClientEvent>,
    ) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let stream = tokio::time::timeout(CONNECT_TIMEOUT, TcpStream::connect(server_addr))
            .await
            .map_err(|_| "Tempo esgotado ao conectar no host")??;
        // Uses the resolved address, since server_addr may be a hostname
        let remote_ip = stream.peer_addr()?.ip();
        let (reader, mut writer) = stream.into_split();
        let mut lines = BufReader::new(reader).lines();

        let udp_socket = Arc::new(UdpSocket::bind("0.0.0.0:0").await?);
        let local_udp_port = udp_socket.local_addr()?.port();

        // 1. Send Join request
        let join_msg = ClientMessage::Join {
            name: name.to_string(),
            room: room.to_string(),
            udp_port: local_udp_port,
        };
        writer
            .write_all(serialize_client_msg(&join_msg).as_bytes())
            .await?;

        // 2. Await Admitted response
        let admitted_line = lines
            .next_line()
            .await?
            .ok_or("Conexão encerrada pelo servidor antes da admissão")?;

        let admitted_msg: ServerMessage = serde_json::from_str(&admitted_line)?;
        let (participant_id, admitted_room, relay_port) = match admitted_msg {
            ServerMessage::Admitted {
                participant_id,
                room,
                udp_relay_port,
            } => (participant_id, room, udp_relay_port),
            ServerMessage::Error { message } => return Err(message.into()),
            _ => return Err("Resposta de admissão inválida".into()),
        };

        let relay_udp_addr = SocketAddr::new(remote_ip, relay_port);

        let (tcp_tx, mut tcp_rx) = mpsc::channel::<String>(32);
        let (shutdown_tx, shutdown_rx) = watch::channel(false);

        // TCP Writer task
        tokio::spawn(async move {
            while let Some(msg) = tcp_rx.recv().await {
                if writer.write_all(msg.as_bytes()).await.is_err() {
                    break;
                }
            }
        });

        // Initialize Audio Playback
        let voice_processor = VoiceProcessor::shared();
        let audio_playback = match AudioPlayback::start(voice_processor.clone()) {
            Ok(playback) => Some(playback),
            Err(e) => {
                let _ = event_tx.send(ClientEvent::Error(format!(
                    "Áudio de saída indisponível: {}",
                    e
                )));
                None
            }
        };

        // Encoded video frames, from the room or from our own preview
        let (video_frame_tx, _) = broadcast::channel(120);
        let screen_receiver = Arc::new(Mutex::new(ScreenReceiver::new(video_frame_tx.clone())));

        // UDP Receiver task
        let udp_rx_socket = udp_socket.clone();
        let mut udp_shutdown_rx = shutdown_rx.clone();
        let rx_audio_buf = audio_playback.as_ref().map(|p| p.clone_buffer_pusher());
        let rx_screen_rec = screen_receiver.clone();

        tokio::spawn(async move {
            let mut buf = vec![0u8; 65535];
            loop {
                tokio::select! {
                    _ = udp_shutdown_rx.changed() => {
                        if *udp_shutdown_rx.borrow() {
                            break;
                        }
                    }
                    res = udp_rx_socket.recv_from(&mut buf) => {
                        if let Ok((len, _)) = res
                            && len > 0
                        {
                            let tag = buf[0];
                            if tag == TAG_AUDIO {
                                if let Some(pusher) = &rx_audio_buf
                                    && len >= 5
                                {
                                    let sender_id = u32::from_be_bytes([buf[1], buf[2], buf[3], buf[4]]);
                                    pusher(sender_id, &buf[5..len]);
                                }
                            } else if tag == TAG_VIDEO {
                                let mut rec = rx_screen_rec.lock().unwrap();
                                rec.handle_packet(&buf[..len]);
                            }
                        }
                    }
                }
            }
        });

        // Setup Audio & Screen Sender channels
        let (audio_packet_tx, mut audio_packet_rx) = mpsc::channel::<Vec<u8>>(64);
        let (screen_packet_tx, mut screen_packet_rx) = mpsc::channel::<Vec<u8>>(1024);
        let (speaking_status_tx, mut speaking_status_rx) = mpsc::channel::<bool>(16);

        // UDP Sender task
        let udp_tx_socket = udp_socket.clone();
        let mut send_shutdown_rx = shutdown_rx.clone();
        let audio_sender_id = fastrand::u32(..);
        tokio::spawn(async move {
            loop {
                tokio::select! {
                    _ = send_shutdown_rx.changed() => {
                        if *send_shutdown_rx.borrow() {
                            break;
                        }
                    }
                    Some(audio_data) = audio_packet_rx.recv() => {
                        let mut packet = Vec::with_capacity(5 + audio_data.len());
                        packet.push(TAG_AUDIO);
                        packet.extend_from_slice(&audio_sender_id.to_be_bytes());
                        packet.extend_from_slice(&audio_data);
                        let _ = udp_tx_socket.send_to(&packet, relay_udp_addr).await;
                    }
                    Some(screen_packet) = screen_packet_rx.recv() => {
                        let _ = udp_tx_socket.send_to(&screen_packet, relay_udp_addr).await;
                    }
                }
            }
        });

        // Speaking status notifier to TCP server
        let speaking_tcp_tx = tcp_tx.clone();
        let mut speaking_shutdown_rx = shutdown_rx.clone();
        tokio::spawn(async move {
            loop {
                tokio::select! {
                    _ = speaking_shutdown_rx.changed() => {
                        if *speaking_shutdown_rx.borrow() {
                            break;
                        }
                    }
                    Some(is_speaking) = speaking_status_rx.recv() => {
                        let msg = ClientMessage::Speaking { is_speaking };
                        let _ = speaking_tcp_tx.send(serialize_client_msg(&msg)).await;
                    }
                }
            }
        });

        // Keepalive so both sides can detect a dead connection
        let ping_tcp_tx = tcp_tx.clone();
        let mut ping_shutdown_rx = shutdown_rx.clone();
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(PING_INTERVAL_SECS));
            loop {
                tokio::select! {
                    _ = ping_shutdown_rx.changed() => break,
                    _ = interval.tick() => {
                        let _ = ping_tcp_tx.try_send(serialize_client_msg(&ClientMessage::Ping));
                    }
                }
            }
        });

        // Start Audio Capture
        let audio_capture =
            match AudioCapture::start(audio_packet_tx, speaking_status_tx, voice_processor) {
                Ok(capture) => Some(capture),
                Err(e) => {
                    let _ =
                        event_tx.send(ClientEvent::Error(format!("Microfone indisponível: {}", e)));
                    None
                }
            };
        let is_mic_enabled = Arc::new(AtomicBool::new(true));

        // TCP Reader task
        let evt_tx = event_tx.clone();
        let mut tcp_shutdown_rx = shutdown_rx.clone();
        tokio::spawn(async move {
            loop {
                tokio::select! {
                    _ = tcp_shutdown_rx.changed() => {
                        if *tcp_shutdown_rx.borrow() {
                            break;
                        }
                    }
                    line_res = tokio::time::timeout(Duration::from_secs(PEER_TIMEOUT_SECS), lines.next_line()) => {
                        match line_res {
                            Ok(Ok(Some(line))) => {
                                if let Ok(msg) = serde_json::from_str::<ServerMessage>(&line) {
                                    match msg {
                                        ServerMessage::RoomState { room, participants, screen_presenter } => {
                                            let _ = evt_tx.send(ClientEvent::RoomStateUpdated {
                                                room,
                                                participants,
                                                screen_presenter,
                                            });
                                        }
                                        ServerMessage::Error { message } => {
                                            let _ = evt_tx.send(ClientEvent::Error(message));
                                        }
                                        _ => {}
                                    }
                                }
                            }
                            _ => {
                                let _ = evt_tx.send(ClientEvent::Disconnected);
                                break;
                            }
                        }
                    }
                }
            }
        });

        Ok(Self {
            participant_id,
            room_name: admitted_room,
            display_name: name.to_string(),
            server_addr: server_addr.to_string(),
            is_host,
            tcp_tx,
            audio_capture,
            audio_playback,
            screen_capture: None,
            screen_packet_tx,
            is_mic_enabled,
            is_deafened: Arc::new(AtomicBool::new(false)),
            is_sharing_screen: Arc::new(AtomicBool::new(false)),
            video_frame_tx,
            shutdown_tx,
        })
    }

    pub fn subscribe_shutdown(&self) -> watch::Receiver<bool> {
        self.shutdown_tx.subscribe()
    }

    pub fn set_microphone(&mut self, enabled: bool) {
        self.is_mic_enabled.store(enabled, Ordering::Relaxed);
        if let Some(cap) = &self.audio_capture {
            cap.is_muted.store(!enabled, Ordering::Relaxed);
        }
    }

    pub fn set_deafened(&mut self, deafened: bool) {
        self.is_deafened.store(deafened, Ordering::Relaxed);
        if let Some(play) = &self.audio_playback {
            play.is_deafened.store(deafened, Ordering::Relaxed);
        }
    }

    pub fn start_screen_share(
        &mut self,
        monitor_idx: usize,
        quality: VideoQuality,
    ) -> Result<(), String> {
        let cap = ScreenCapture::start(
            monitor_idx,
            quality,
            self.screen_packet_tx.clone(),
            self.video_frame_tx.clone(),
        )
        .map_err(|e| format!("Falha ao iniciar compartilhamento: {}", e))?;
        self.screen_capture = Some(cap);
        self.is_sharing_screen.store(true, Ordering::Relaxed);

        let msg = ClientMessage::ScreenShareStarted {
            monitor_name: format!("Monitor {}", monitor_idx + 1),
        };
        let _ = self.tcp_tx.try_send(serialize_client_msg(&msg));
        Ok(())
    }

    pub fn stop_screen_share(&mut self) {
        if let Some(cap) = self.screen_capture.take() {
            cap.stop();
        }
        self.is_sharing_screen.store(false, Ordering::Relaxed);
        let msg = ClientMessage::ScreenShareStopped;
        let _ = self.tcp_tx.try_send(serialize_client_msg(&msg));
    }

    pub fn disconnect(&mut self) {
        self.stop_screen_share();
        let _ = self
            .tcp_tx
            .try_send(serialize_client_msg(&ClientMessage::Leave));
        let _ = self.shutdown_tx.send(true);
    }
}
