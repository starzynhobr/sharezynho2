use serde::{Deserialize, Serialize};

pub const DEFAULT_TCP_PORT: u16 = 9765;
pub const DEFAULT_UDP_PORT: u16 = 9766;

pub const TAG_AUDIO: u8 = 0x01;
pub const TAG_VIDEO: u8 = 0x02;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParticipantInfo {
    pub id: String,
    pub name: String,
    pub is_speaking: bool,
    pub is_host: bool,
    pub is_sharing_screen: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "payload")]
pub enum ClientMessage {
    Join {
        name: String,
        room: String,
        udp_port: u16,
    },
    Leave,
    Speaking {
        is_speaking: bool,
    },
    ScreenShareStarted {
        monitor_name: String,
    },
    ScreenShareStopped,
    Ping,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "payload")]
pub enum ServerMessage {
    Admitted {
        participant_id: String,
        room: String,
        udp_relay_port: u16,
    },
    RoomState {
        room: String,
        participants: Vec<ParticipantInfo>,
        screen_presenter: Option<String>,
    },
    Error {
        message: String,
    },
    Pong,
}

/// Helper to serialize a ServerMessage into a line-delimited JSON string.
pub fn serialize_server_msg(msg: &ServerMessage) -> String {
    let mut s = serde_json::to_string(msg).unwrap_or_default();
    s.push('\n');
    s
}

/// Helper to serialize a ClientMessage into a line-delimited JSON string.
pub fn serialize_client_msg(msg: &ClientMessage) -> String {
    let mut s = serde_json::to_string(msg).unwrap_or_default();
    s.push('\n');
    s
}
