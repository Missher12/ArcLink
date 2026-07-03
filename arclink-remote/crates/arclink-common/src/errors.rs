use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProtocolError {
    InvalidMagic,
    MessageTooLarge,
    SerializationFailed(String),
    DeserializationFailed(String),
    ConnectionReset,
    UnexpectedMessage,
    VersionMismatch,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SessionError {
    NotAuthorized,
    SessionExpired,
    SessionNotFound,
    AlreadyConnected,
    HostRejected(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CaptureError {
    InitFailed(String),
    CaptureFailed(String),
    UnsupportedPlatform,
    DeviceLost,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum InputError {
    InjectionFailed(String),
    PermissionDenied,
    InvalidCoordinates,
}
