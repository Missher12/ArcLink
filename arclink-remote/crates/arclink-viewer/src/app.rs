use crate::logging::ViewerLogger;
use crate::metrics::ViewerMetricsManager;
use crate::network::NetworkManager;
use crate::render::VideoFrameBuffer;
use crate::session::ViewerSessionManager;
use crate::ui::ViewerUi;

use eframe::egui;
use std::sync::{Arc, Mutex};

pub struct ViewerApp {
    net_mgr: Arc<Mutex<NetworkManager>>,
    logger: Arc<Mutex<ViewerLogger>>,
    frame_buffer: Arc<Mutex<VideoFrameBuffer>>,
    ui_renderer: ViewerUi,
}

impl ViewerApp {
    pub fn new() -> Self {
        let logger = Arc::new(Mutex::new(ViewerLogger::new()));
        logger.lock().unwrap().info("ArcLink Viewer Client Booting...");

        let metrics_mgr = ViewerMetricsManager::new();
        let session_mgr = ViewerSessionManager::new();
        let frame_buffer = Arc::new(Mutex::new(VideoFrameBuffer::new()));
        
        let net_mgr = Arc::new(Mutex::new(NetworkManager::new(
            metrics_mgr,
            session_mgr,
            frame_buffer.clone(),
        )));

        Self {
            net_mgr,
            logger,
            frame_buffer,
            ui_renderer: ViewerUi::new(),
        }
    }
}

impl eframe::App for ViewerApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Run UI drawing
        self.ui_renderer.draw(ctx, &self.net_mgr, &self.logger, &self.frame_buffer);

        // Request repaint to render active remote frame streams smoothly
        ctx.request_repaint_after(std::time::Duration::from_millis(16)); // ~60 FPS update cycle
    }
}
