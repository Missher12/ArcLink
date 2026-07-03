use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc};
use crate::metrics::SessionMetrics;
use crate::session::DisconnectReason;

/// Magic bytes for the framing layer
pub const PROTOCOL_MAGIC: [u8; 4] = [0x41, 0x52, 0x43, 0x4C]; // "ARCL"

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HelloMessage {
    pub protocol_version: String,
    pub sender_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionRequest {
    pub session_id: String,
    pub viewer_name: String,
    pub viewer_ip: String,
    pub request_time: DateTime<Utc>,
    pub required_fps: u32,
    pub width: u32,
    pub height: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionAccept {
    pub session_id: String,
    pub host_name: String,
    pub accepted_time: DateTime<Utc>,
    pub control_port: u16,
    pub video_port: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionReject {
    pub session_id: String,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Heartbeat {
    pub timestamp_us: u64,
    pub sequence: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeartbeatAck {
    pub timestamp_us: u64,
    pub sequence: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DisconnectMessage {
    pub reason: DisconnectReason,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProtocolErrorMessage {
    pub code: u16,
    pub description: String,
}

/// Control message enumeration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ControlMessage {
    Hello(HelloMessage),
    SessionRequest(SessionRequest),
    SessionAccept(SessionAccept),
    SessionReject(SessionReject),
    Heartbeat(Heartbeat),
    HeartbeatAck(HeartbeatAck),
    SessionMetrics(SessionMetrics),
    Disconnect(DisconnectMessage),
    Error(ProtocolErrorMessage),
}

/// Video packet header for UDP streaming
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub struct VideoPacketHeader {
    pub magic: u32,
    pub protocol_version: u16,
    pub session_id: u128,
    pub frame_id: u64,
    pub capture_timestamp_us: u64,
    pub fragment_index: u16,
    pub fragment_count: u16,
    pub payload_len: u16,
    pub flags: u16,
}
