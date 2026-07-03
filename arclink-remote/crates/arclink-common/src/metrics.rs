use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionMetrics {
    pub control_rtt_ms: Option<f32>,
    pub input_rtt_ms: Option<f32>,
    pub capture_fps: Option<f32>,
    pub render_fps: Option<f32>,
    pub bitrate_kbps: Option<f32>,
    pub sent_bytes_per_sec: Option<f64>,
    pub received_bytes_per_sec: Option<f64>,
    pub packet_loss_rate: Option<f32>,
    pub jitter_ms: Option<f32>,
    pub capture_latency_ms: Option<f32>,
    pub render_latency_ms: Option<f32>,
    pub active_duration_secs: u64,
}

impl Default for SessionMetrics {
    fn default() -> Self {
        Self {
            control_rtt_ms: None,
            input_rtt_ms: None,
            capture_fps: None,
            render_fps: None,
            bitrate_kbps: None,
            sent_bytes_per_sec: None,
            received_bytes_per_sec: None,
            packet_loss_rate: None,
            jitter_ms: None,
            capture_latency_ms: None,
            render_latency_ms: None,
            active_duration_secs: 0,
        }
    }
}
