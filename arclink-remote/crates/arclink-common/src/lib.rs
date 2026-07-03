pub mod errors;
pub mod framing;
pub mod input;
pub mod metrics;
pub mod protocol;
pub mod session;

// Re-export common models and types for convenience
pub use errors::{ProtocolError, SessionError, CaptureError, InputError};
pub use framing::{encode_frame, decode_payload, read_frame_sync, write_frame_sync};
pub use input::{
    InputEvent, MouseButton, MouseButtonEvent, MouseMoveEvent, MouseWheelEvent, KeyboardEvent,
};
pub use metrics::SessionMetrics;
pub use protocol::{
    HelloMessage, SessionRequest, SessionAccept, SessionReject, Heartbeat, HeartbeatAck,
    DisconnectMessage, ProtocolErrorMessage, ControlMessage, VideoPacketHeader, PROTOCOL_MAGIC,
};
pub use session::{DeviceInfo, DisconnectReason, HostStatus, RemoteSession, ViewerStatus};
