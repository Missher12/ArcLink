use std::sync::{Arc, Mutex};
use std::time::Instant;
use eframe::egui;
use chrono::Utc;
use arclink_common::{
    ViewerStatus, InputEvent, MouseMoveEvent, MouseButtonEvent, MouseButton, KeyboardEvent,
    SessionRequest, SessionAccept, SessionReject, ControlMessage, DisconnectReason, SessionMetrics, NetworkStats
};

pub struct ViewerApp {
    host_ip: String,
    host_port: String,
    device_remark: String,
    status: ViewerStatus,
    metrics: Arc<Mutex<SessionMetrics>>,
    logs: Vec<String>,
    show_perf_panel: bool,
    input_locked: bool,
    quality_mode: String,
    last_ping: Instant,
    simulated_cursor_x: f32,
    simulated_cursor_y: f32,
}

impl ViewerApp {
    pub fn new() -> Self {
        let metrics = Arc::new(Mutex::new(SessionMetrics {
            latency_ms: 8.5,
            fps: 60,
            bitrate_kbps: 8500.2,
            resolution_width: 1920,
            resolution_height: 1080,
            active_duration_secs: 145,
            network: NetworkStats {
                rtt_ms: 1.5,
                jitter_ms: 0.2,
                packet_loss_rate: 0.0,
                sent_bytes_sec: 2048.0,
                rcv_bytes_sec: 1048576.0,
            },
        }));

        let mut app = Self {
            host_ip: "192.168.1.100".to_string(),
            host_port: "8443".to_string(),
            device_remark: "Remote Workstation A".to_string(),
            status: ViewerStatus::Disconnected,
            metrics,
            logs: Vec::new(),
            show_perf_panel: false,
            input_locked: false,
            quality_mode: "Smooth (60fps)".to_string(),
            last_ping: Instant::now(),
            simulated_cursor_x: 960.0,
            simulated_cursor_y: 540.0,
        };

        app.log("ArcLink Viewer initialized.");
        app
    }

    fn log(&mut self, msg: &str) {
        let time = Utc::now().format("%H:%M:%S").to_string();
        self.logs.push(format!("[{}] {}", time, msg));
        if self.logs.len() > 30 {
            self.logs.remove(0);
        }
    }
}

impl eframe::App for ViewerApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Light clean apple liquid glass style theme
        let mut visuals = egui::Visuals::light();
        visuals.widgets.noninteractive.bg_fill = egui::Color32::from_rgb(0xF5, 0xF7, 0xFA);
        visuals.widgets.inactive.bg_fill = egui::Color32::from_rgba_unmultiplied(255, 255, 255, 200);
        visuals.window_rounding = 12.0.into();
        ctx.set_visuals(visuals);

        if self.status == ViewerStatus::Disconnected || self.status == ViewerStatus::Connecting {
            // LOGIN / CONNECTION SCREEN
            egui::CentralPanel::default().show(ctx, |ui| {
                ui.add_space(20.0);
                ui.vertical_central(|ui| {
                    ui.label(egui::RichText::new("ArcLink Viewer").font(egui::FontId::proportional(28.0)).strong());
                    ui.label(egui::RichText::new("LAN Remote Desktop Controller").color(egui::Color32::from_rgb(120, 130, 145)));
                });

                ui.add_space(24.0);

                // Connection box
                egui::Frame::none()
                    .fill(egui::Color32::from_rgba_unmultiplied(255, 255, 255, 230))
                    .stroke(egui::Stroke::new(1.0, egui::Color32::from_rgb(220, 225, 235)))
                    .rounding(12.0)
                    .inner_margin(18.0)
                    .show(ui, |ui| {
                        ui.set_max_width(360.0);
                        ui.vertical(|ui| {
                            ui.label(egui::RichText::new("Connect to a Host Device").font(egui::FontId::proportional(16.0)).strong());
                            ui.add_space(4.0);
                            ui.label("Input local IP and Port of ArcLink Host:");
                            ui.add_space(10.0);

                            ui.horizontal(|ui| {
                                ui.label("Host IP Address:");
                            });
                            ui.text_edit_singleline(&mut self.host_ip);
                            ui.add_space(6.0);

                            ui.horizontal(|ui| {
                                ui.label("Port number:");
                            });
                            ui.text_edit_singleline(&mut self.host_port);
                            ui.add_space(6.0);

                            ui.horizontal(|ui| {
                                ui.label("Remark Name (Optional):");
                            });
                            ui.text_edit_singleline(&mut self.device_remark);

                            ui.add_space(14.0);

                            let action_text = if self.status == ViewerStatus::Connecting { "Connecting..." } else { "Establish Connection" };
                            let conn_btn = ui.add_sized([ui.available_width(), 32.0], egui::Button::new(action_text)
                                .fill(egui::Color32::from_rgb(59, 130, 246)));

                            if conn_btn.clicked() && self.status != ViewerStatus::Connecting {
                                self.status = ViewerStatus::Connecting;
                                self.log(&format!("Dialing socket {}:{}...", self.host_ip, self.host_port));
                            }
                        });
                    });

                if self.status == ViewerStatus::Connecting {
                    ui.add_space(15.0);
                    ui.horizontal(|ui| {
                        ui.spinner();
                        ui.label("Connecting, waiting for host acceptance...");
                        if ui.button("Cancel").clicked() {
                            self.status = ViewerStatus::Disconnected;
                            self.log("Connection aborted by user.");
                        }
                    });

                    // Simulated handshake success for testing
                    ui.add_space(8.0);
                    if ui.button("Simulate Host Accepted Connection").clicked() {
                        self.status = ViewerStatus::Connected;
                        self.log("TCP control pipeline established. UDP video frame rendering started.");
                    }
                }

                ui.add_space(20.0);
                ui.label("Recent Connections:");
                egui::Frame::none()
                    .fill(egui::Color32::from_rgba_unmultiplied(255, 255, 255, 120))
                    .rounding(6.0)
                    .inner_margin(8.0)
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            ui.label("🖥️ 192.168.1.100 : 8443");
                            ui.separator();
                            ui.label("Workstation A");
                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                if ui.small_button("Connect").clicked() {
                                    self.host_ip = "192.168.1.100".to_string();
                                    self.host_port = "8443".to_string();
                                    self.status = ViewerStatus::Connecting;
                                }
                            });
                        });
                    });
            });
        } else {
            // REMOTE CONTROL ACTIVE SESSION VIEW
            egui::TopBottomPanel::top("control_bar").show(ctx, |ui| {
                ui.horizontal(|ui| {
                    if ui.button("↩ Exit Session").clicked() {
                        self.status = ViewerStatus::Disconnected;
                        self.log("Remote control session closed by user.");
                    }
                    ui.separator();
                    ui.label(egui::RichText::new(&self.device_remark).strong());
                    ui.label(format!("({}:{})", self.host_ip, self.host_port));
                    ui.separator();

                    let m = self.metrics.lock().unwrap();
                    ui.label(format!("FPS: {}", m.fps));
                    ui.label("•");
                    ui.label(format!("RTT: {:.1} ms", m.network.rtt_ms));
                    ui.label("•");
                    ui.label(format!("Bitrate: {:.0} kbps", m.bitrate_kbps));

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.button(if self.show_perf_panel { "Hide Stats 📊" } else { "Show Stats 📊" }).clicked() {
                            self.show_perf_panel = !self.show_perf_panel;
                        }
                        
                        ui.checkbox(&mut self.input_locked, "Lock Keyboard");
                    });
                });
            });

            if self.show_perf_panel {
                egui::SidePanel::right("perf_panel").show(ctx, |ui| {
                    ui.vertical(|ui| {
                        ui.heading("Session Diagnostics");
                        ui.add_space(8.0);

                        let m = self.metrics.lock().unwrap();
                        ui.label("Real-time network indices:");
                        ui.add_space(4.0);

                        egui::Grid::new("perf_grid").num_columns(2).spacing([12.0, 8.0]).striped(true).show(ui, |ui| {
                            ui.label("Latency:");
                            ui.label(format!("{:.2} ms", m.latency_ms));
                            ui.end_row();

                            ui.label("FPS Output:");
                            ui.label(format!("{} frames/sec", m.fps));
                            ui.end_row();

                            ui.label("Downstream BW:");
                            ui.label(format!("{:.1} Kbps", m.bitrate_kbps));
                            ui.end_row();

                            ui.label("Packet Loss:");
                            ui.label(format!("{:.3}%", m.network.packet_loss_rate * 100.0));
                            ui.end_row();

                            ui.label("Connection Type:");
                            ui.label("LAN Direct");
                            ui.end_row();
                        });

                        ui.add_space(16.0);
                        ui.label("Quality Modes:");
                        if ui.selectable_label(self.quality_mode == "Smooth (60fps)", "Smooth (60fps)").clicked() {
                            self.quality_mode = "Smooth (60fps)".to_string();
                        }
                        if ui.selectable_label(self.quality_mode == "Retina Clear", "Retina Clear").clicked() {
                            self.quality_mode = "Retina Clear".to_string();
                        }
                        if ui.selectable_label(self.quality_mode == "Eco-Bandwidth", "Eco-Bandwidth").clicked() {
                            self.quality_mode = "Eco-Bandwidth".to_string();
                        }
                    });
                });
            }

            egui::CentralPanel::default().show(ctx, |ui| {
                // RENDER REMOTE DESKTOP CANVAS
                let response = egui::Frame::none()
                    .fill(egui::Color32::from_rgb(30, 32, 35))
                    .rounding(8.0)
                    .inner_margin(12.0)
                    .show(ui, |ui| {
                        let rect = ui.available_rect_before_wrap();
                        
                        // Handle user pointer interactions to simulate remote mouse control
                        if ui.rect_contains_pointer(rect) {
                            if let Some(pos) = ui.ctx().pointer_latest_pos() {
                                // Map to coordinates
                                let rx = pos.x - rect.min.x;
                                let ry = pos.y - rect.min.y;
                                self.simulated_cursor_x = (rx / rect.width() * 1920.0).clamp(0.0, 1920.0);
                                self.simulated_cursor_y = (ry / rect.height() * 1080.0).clamp(0.0, 1080.0);
                            }
                        }

                        // Draw a beautiful simulated Windows 11 Desktop inside the panel
                        let painter = ui.painter();
                        
                        // Main background gradient color of remote desktop
                        painter.rect_filled(rect, 4.0, egui::Color32::from_rgb(26, 54, 93));

                        // Draw simulated taskbar at the bottom
                        let taskbar_h = 24.0;
                        let taskbar_rect = egui::Rect::from_min_max(
                            egui::pos2(rect.min.x, rect.max.y - taskbar_h),
                            rect.max
                        );
                        painter.rect_filled(taskbar_rect, 0.0, egui::Color32::from_rgba_unmultiplied(20, 25, 35, 230));

                        // Center taskbar icons
                        let tb_center = taskbar_rect.center().x;
                        painter.circle_filled(egui::pos2(tb_center - 15.0, taskbar_rect.center().y), 6.0, egui::Color32::LIGHT_BLUE);
                        painter.circle_filled(egui::pos2(tb_center, taskbar_rect.center().y), 6.0, egui::Color32::WHITE);
                        painter.circle_filled(egui::pos2(tb_center + 15.0, taskbar_rect.center().y), 6.0, egui::Color32::LIGHT_GRAY);

                        // Draw simulated remote window
                        let window_rect = egui::Rect::from_min_max(
                            egui::pos2(rect.min.x + 40.0, rect.min.y + 30.0),
                            egui::pos2(rect.min.x + 300.0, rect.min.y + 200.0)
                        );
                        painter.rect_filled(window_rect, 8.0, egui::Color32::from_rgba_unmultiplied(255, 255, 255, 240));
                        painter.text(
                            egui::pos2(rect.min.x + 50.0, rect.min.y + 50.0),
                            egui::Align2::LEFT_TOP,
                            "File Explorer",
                            egui::FontId::proportional(14.0),
                            egui::Color32::from_rgb(20, 30, 45)
                        );

                        // Draw mouse coordinates
                        painter.text(
                            egui::pos2(rect.min.x + 12.0, rect.min.y + 12.0),
                            egui::Align2::LEFT_TOP,
                            format!("Mapped Cursor: ({:.0}, {:.0})", self.simulated_cursor_x, self.simulated_cursor_y),
                            egui::FontId::monospace(11.0),
                            egui::Color32::LIGHT_GREEN
                        );

                        // Draw simulated mouse cursor dot on host screen
                        let cursor_pos_screen = egui::pos2(
                            rect.min.x + (self.simulated_cursor_x / 1920.0) * rect.width(),
                            rect.min.y + (self.simulated_cursor_y / 1080.0) * rect.height()
                        );
                        painter.circle_filled(cursor_pos_screen, 4.0, egui::Color32::WHITE);
                        painter.circle_stroke(cursor_pos_screen, 6.0, egui::Stroke::new(1.0, egui::Color32::BLACK));
                    });
            });
        }
    }
}

fn main() -> Result<(), eframe::Error> {
    tracing_subscriber::fmt::init();
    
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([720.0, 500.0]),
        ..Default::default()
    };
    
    eframe::run_native(
        "ArcLink Viewer",
        options,
        Box::new(|_cc| Box::new(ViewerApp::new())),
    )
}
