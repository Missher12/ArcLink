use arclink_common::SessionMetrics;
use std::sync::{Arc, Mutex};
use std::time::Instant;

#[derive(Clone)]
pub struct ViewerMetricsManager {
    metrics: Arc<Mutex<SessionMetrics>>,
    start_time: Option<Instant>,
    bytes_recv_accum: Arc<Mutex<u64>>,
    last_bitrate_check: Arc<Mutex<Instant>>,
}

impl ViewerMetricsManager {
    pub fn new() -> Self {
        Self {
            metrics: Arc::new(Mutex::new(SessionMetrics::default())),
            start_time: None,
            bytes_recv_accum: Arc::new(Mutex::new(0)),
            last_bitrate_check: Arc::new(Mutex::new(Instant::now())),
        }
    }

    pub fn start_session(&mut self) {
        self.start_time = Some(Instant::now());
        let mut m = self.metrics.lock().unwrap();
        *m = SessionMetrics::default();
    }

    pub fn stop_session(&mut self) {
        self.start_time = None;
        let mut m = self.metrics.lock().unwrap();
        *m = SessionMetrics::default();
    }

    pub fn update_rtt(&self, rtt_ms: f32) {
        let mut m = self.metrics.lock().unwrap();
        m.control_rtt_ms = Some(rtt_ms);
    }

    pub fn update_render_fps(&self, fps: f32) {
        let mut m = self.metrics.lock().unwrap();
        m.render_fps = Some(fps);
    }

    pub fn update_render_latency(&self, latency_ms: f32) {
        let mut m = self.metrics.lock().unwrap();
        m.render_latency_ms = Some(latency_ms);
    }

    pub fn add_bytes_recv(&self, bytes: usize) {
        let mut accum = self.bytes_recv_accum.lock().unwrap();
        *accum += bytes as u64;

        let mut last_check = self.last_bitrate_check.lock().unwrap();
        let elapsed = last_check.elapsed();
        if elapsed.as_secs_f32() >= 1.0 {
            let recv = *accum;
            *accum = 0;
            *last_check = Instant::now();

            let mut m = self.metrics.lock().unwrap();
            let bytes_sec = recv as f64 / elapsed.as_secs_f64();
            m.received_bytes_per_sec = Some(bytes_sec);
            m.bitrate_kbps = Some((bytes_sec * 8.0 / 1000.0) as f32);
        }
    }

    pub fn get_metrics(&self) -> SessionMetrics {
        let mut m = self.metrics.lock().unwrap().clone();
        if let Some(start) = self.start_time {
            m.active_duration_secs = start.elapsed().as_secs();
        }
        m
    }
}
