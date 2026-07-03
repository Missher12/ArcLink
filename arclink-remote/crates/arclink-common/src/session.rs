use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceInfo {
    pub device_id: String,
    pub device_name: String,
    pub local_ip: String,
    pub listen_port: u16,
    pub screen_width: u32,
    pub screen_height: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HostStatus {
    Idle,
    Listening,
    Connecting,
    Occupied,
    Error,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ViewerStatus {
    Disconnected,
    Connecting,
    Connected,
    Reconnecting,
    Error,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DisconnectReason {
    UserClosed,
    HostRejected,
    HostDisconnected,
    Timeout,
    NetworkError,
    ProtocolViolation,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemoteSession {
    pub session_id: String,
    pub viewer_id: String,
    pub host_id: String,
    pub viewer_ip: String,
    pub host_ip: String,
    pub start_time: DateTime<Utc>,
    pub protocol_version: String,
    pub allow_control: bool,
}
