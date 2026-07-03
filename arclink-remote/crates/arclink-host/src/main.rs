pub mod app;
pub mod capture;
pub mod input;
pub mod logging;
pub mod metrics;
pub mod network;
pub mod session;
pub mod ui;

use eframe::egui;

fn main() -> Result<(), eframe::Error> {
    // Initialize standard tracing
    tracing_subscriber::fmt::init();

    // Fire up a multi-threaded Tokio runtime for background sockets handling
    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .expect("Failed to build multi-threaded Tokio runtime");
    let _guard = rt.enter();

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([460.0, 540.0])
            .with_resizable(false),
        ..Default::default()
    };

    eframe::run_native(
        "ArcLink Host",
        options,
        Box::new(|_cc| Box::new(crate::app::HostApp::new())),
    )
}
