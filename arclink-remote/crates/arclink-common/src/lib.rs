use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc};

/// Device information details
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceInfo {
    pub device_id: String,
    pub device_name: String,
    pub local_ip: String,
    pub listen_port: u16,
    pub screen_width: u32,
    pub screen_height: u32,
}

/// Current status of the Host
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HostStatus {
    Idle,
    Listening,
    Connecting,
    Occupied,
    Error,
}

/// Current status of the Viewer
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ViewerStatus {
    Disconnected,
    Connecting,
    Connected,
    Reconnecting,
    Error,
}

/// Disconnect reasons for a session
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DisconnectReason {
    UserClosed,
    HostRejected,
    HostDisconnected,
    Timeout,
    NetworkError,
    ProtocolViolation,
}

/// Session parameters and connection info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemoteSession {
    pub session_id: String,
    pub viewer_ip: String,
    pub host_ip: String,
    pub start_time: DateTime<Utc>,
    pub protocol_version: String,
}

/// Connection request sent from Viewer to Host
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

/// Accept response from Host to Viewer
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionAccept {
    pub session_id: String,
    pub host_name: String,
    pub accepted_time: DateTime<Utc>,
    pub control_port: u16,
    pub video_port: u16,
}

/// Reject response from Host to Viewer
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionReject {
    pub session_id: String,
    pub reason: String,
}

/// Network metrics measured in real-time
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkStats {
    pub rtt_ms: f32,
    pub jitter_ms: f32,
    pub packet_loss_rate: f32,
    pub sent_bytes_sec: f32,
    pub rcv_bytes_sec: f32,
}

/// Performance and session metrics combined
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionMetrics {
    pub latency_ms: f32,
    pub fps: u32,
    pub bitrate_kbps: f32,
    pub resolution_width: u32,
    pub resolution_height: u32,
    pub active_duration_secs: u64,
    pub network: NetworkStats,
}

/// Video frame representation (compressed payload)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VideoFrame {
    pub frame_index: u64,
    pub timestamp: DateTime<Utc>,
    pub width: u32,
    pub height: u32,
    /// Encoded image payload (JPEG/H.264/H.265/VP9 bytes)
    pub payload: Vec<u8>,
    pub is_keyframe: bool,
    pub encoding_duration_ms: f32,
}

/// Heartbeat structure for connection liveness
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Heartbeat {
    pub timestamp: DateTime<Utc>,
    pub sequence: u64,
}

/// Mouse button identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MouseButton {
    Left,
    Right,
    Middle,
}

/// Mouse movement event structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MouseMoveEvent {
    /// Normalized coordinates between 0.0 and 1.0 (to fit host screen dynamically)
    pub norm_x: f32,
    pub norm_y: f32,
}

/// Mouse button click or release event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MouseButtonEvent {
    pub button: MouseButton,
    pub is_down: bool,
}

/// Mouse wheel scroll event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MouseWheelEvent {
    pub delta_x: f32,
    pub delta_y: f32,
}

/// Keyboard key action event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyboardEvent {
    pub vk_code: u16, // Virtual key code
    pub is_down: bool,
    pub modifiers: u8, // Flags: Shift, Ctrl, Alt, Win
}

/// Universal input event wrappers
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum InputEvent {
    MouseMove(MouseMoveEvent),
    MouseButton(MouseButtonEvent),
    MouseWheel(MouseWheelEvent),
    Keyboard(KeyboardEvent),
}

/// Protocol Control Channel Message types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ControlMessage {
    Request(SessionRequest),
    Accept(SessionAccept),
    Reject(SessionReject),
    Disconnect(DisconnectReason),
    Heartbeat(Heartbeat),
    Stats(SessionMetrics),
}
