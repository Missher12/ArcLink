use crate::logging::HostLogger;
use crate::metrics::HostMetricsManager;
use crate::network::NetworkManager;
use crate::session::HostSessionManager;
use crate::ui::HostUi;

use eframe::egui;
use std::sync::{Arc, Mutex};

pub struct HostApp {
    net_mgr: Arc<Mutex<NetworkManager>>,
    logger: Arc<Mutex<HostLogger>>,
    ui_renderer: HostUi,
}

impl HostApp {
    pub fn new() -> Self {
        let logger = Arc::new(Mutex::new(HostLogger::new()));
        logger.lock().unwrap().info("ArcLink Host Service Booting...");

        let metrics_mgr = HostMetricsManager::new();
        let session_mgr = HostSessionManager::new();
        let net_mgr = Arc::new(Mutex::new(NetworkManager::new(metrics_mgr, session_mgr)));

        Self {
            net_mgr,
            logger,
            ui_renderer: HostUi::new(),
        }
    }
}

impl eframe::App for HostApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Run UI drawing
        self.ui_renderer.draw(ctx, &self.net_mgr, &self.logger);

        // Periodically request repaint (about 30 times a second) to ensure active metrics update immediately
        ctx.request_repaint_after(std::time::Duration::from_millis(33));
    }
}
