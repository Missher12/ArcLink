use std::sync::{Arc, Mutex};
use std::net::SocketAddr;
use std::time::{Duration, Instant};
use eframe::egui;
use chrono::Utc;
use arclink_common::{
    DeviceInfo, HostStatus, InputEvent, MouseButton, MouseMoveEvent, KeyboardEvent,
    SessionRequest, SessionAccept, SessionReject, ControlMessage, DisconnectReason, SessionMetrics
};

/// Abstract trait for screen capture
pub trait ScreenCapturer: Send + Sync {
    fn capture_frame(&mut self) -> Result<Vec<u8>, String>;
    fn name(&self) -> &str;
}

/// Fallback GDI or Simulated Capturer (for cross-platform compiling and MVP)
pub struct LocalMockCapturer {
    counter: u64,
}

impl ScreenCapturer for LocalMockCapturer {
    fn capture_frame(&mut self) -> Result<Vec<u8>, String> {
        self.counter += 1;
        // Generate a simple simulated jpeg bytes or mock payload
        Ok(vec![0xAA, 0xBB, 0xCC, 0xDD, self.counter as u8])
    }
    fn name(&self) -> &str {
        "Software Screen Capturer (Fallback)"
    }
}

/// Input injector abstraction
pub struct InputInjector;

impl InputInjector {
    pub fn new() -> Self {
        Self
    }

    pub fn inject_event(&self, event: &InputEvent) -> Result<(), String> {
        #[cfg(target_os = "windows")]
        {
            use windows::Win32::UI::Input::KeyboardAndMouse::*;
            match event {
                InputEvent::MouseMove(mv) => {
                    // Map 0.0-1.0 to Windows absolute input coordinates (0-65535)
                    let x = (mv.norm_x * 65535.0) as i32;
                    let y = (mv.norm_y * 65535.0) as i32;
                    let mut input = INPUT {
                        r#type: INPUT_MOUSE,
                        Anonymous: INPUT_0 {
                            mi: MOUSEINPUT {
                                dx: x,
                                dy: y,
                                mouseData: 0,
                                dwFlags: MOUSEEVENTF_ABSOLUTE | MOUSEEVENTF_MOVE,
                                time: 0,
                                dwExtraInfo: 0,
                            },
                        },
                    };
                    unsafe { SendInput(&[input], std::mem::size_of::<INPUT>() as i32) };
                }
                InputEvent::MouseButton(mb) => {
                    let flags = match (mb.button, mb.is_down) {
                        (MouseButton::Left, true) => MOUSEEVENTF_LEFTDOWN,
                        (MouseButton::Left, false) => MOUSEEVENTF_LEFTUP,
                        (MouseButton::Right, true) => MOUSEEVENTF_RIGHTDOWN,
                        (MouseButton::Right, false) => MOUSEEVENTF_RIGHTUP,
                        (MouseButton::Middle, true) => MOUSEEVENTF_MIDDLEDOWN,
                        (MouseButton::Middle, false) => MOUSEEVENTF_MIDDLEUP,
                    };
                    let input = INPUT {
                        r#type: INPUT_MOUSE,
                        Anonymous: INPUT_0 {
                            mi: MOUSEINPUT {
                                dx: 0,
                                dy: 0,
                                mouseData: 0,
                                dwFlags: flags,
                                time: 0,
                                dwExtraInfo: 0,
                            },
                        },
                    };
                    unsafe { SendInput(&[input], std::mem::size_of::<INPUT>() as i32) };
                }
                InputEvent::Keyboard(kb) => {
                    let flags = if kb.is_down { KEYBD_EVENT_FLAGS(0) } else { KEYEVENTF_KEYUP };
                    let input = INPUT {
                        r#type: INPUT_KEYBOARD,
                        Anonymous: INPUT_0 {
                            ki: KEYBDINPUT {
                                wVk: VIRTUAL_KEY(kb.vk_code),
                                wScan: 0,
                                dwFlags: flags,
                                time: 0,
                                dwExtraInfo: 0,
                            },
                        },
                    };
                    unsafe { SendInput(&[input], std::mem::size_of::<INPUT>() as i32) };
                }
                _ => {}
            }
            Ok(())
        }
        #[cfg(not(target_os = "windows"))]
        {
            println!("Simulating input injection on this OS: {:?}", event);
            Ok(())
        }
    }
}

pub struct HostApp {
    device_info: DeviceInfo,
    status: HostStatus,
    allow_control: bool,
    metrics: Arc<Mutex<SessionMetrics>>,
    injector: Arc<InputInjector>,
    capturer: Box<dyn ScreenCapturer>,
    active_request: Option<SessionRequest>,
    active_session_client_ip: Option<String>,
    session_start_time: Option<Instant>,
    logs: Vec<String>,
}

impl HostApp {
    pub fn new() -> Self {
        let device_info = DeviceInfo {
            device_id: "ARC-HOST-7890".to_string(),
            device_name: "WORKSTATION-WIN11-PRO".to_string(),
            local_ip: "192.168.1.100".to_string(),
            listen_port: 8443,
            screen_width: 1920,
            screen_height: 1080,
        };

        let metrics = Arc::new(Mutex::new(SessionMetrics {
            latency_ms: 12.4,
            fps: 30,
            bitrate_kbps: 4200.5,
            resolution_width: 1920,
            resolution_height: 1080,
            active_duration_secs: 0,
            network: arclink_common::NetworkStats {
                rtt_ms: 2.1,
                jitter_ms: 0.4,
                packet_loss_rate: 0.001,
                sent_bytes_sec: 524288.0,
                rcv_bytes_sec: 1024.0,
            },
        }));

        let mut app = Self {
            device_info,
            status: HostStatus::Idle,
            allow_control: true,
            metrics,
            injector: Arc::new(InputInjector::new()),
            capturer: Box::new(LocalMockCapturer { counter: 0 }),
            active_request: None,
            active_session_client_ip: None,
            session_start_time: None,
            logs: Vec::new(),
        };

        app.log("ArcLink Host Service initialized.");
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

impl eframe::App for HostApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Apply Apple Liquid Glass / clean light visual theme
        let mut visuals = egui::Visuals::light();
        visuals.widgets.noninteractive.bg_fill = egui::Color32::from_rgb(0xF5, 0xF7, 0xFA); // Main background
        visuals.widgets.inactive.bg_fill = egui::Color32::from_rgba_unmultiplied(255, 255, 255, 180); // Translucent glass input
        visuals.widgets.hovered.bg_fill = egui::Color32::from_rgba_unmultiplied(230, 240, 255, 200); // Hover light blue
        visuals.widgets.active.bg_fill = egui::Color32::from_rgb(0xD0, 0xE0, 0xFF); // Active state
        visuals.window_rounding = 12.0.into();
        ctx.set_visuals(visuals);

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.add_space(8.0);
            
            // Header
            ui.horizontal(|ui| {
                ui.heading(egui::RichText::new("ArcLink Host").font(egui::FontId::proportional(22.0)).strong());
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    match self.status {
                        HostStatus::Idle => {
                            ui.label(egui::RichText::new("● Idle").color(egui::Color32::from_rgb(120, 120, 120)));
                        }
                        HostStatus::Listening => {
                            ui.label(egui::RichText::new("● Listening").color(egui::Color32::from_rgb(34, 197, 94)));
                        }
                        HostStatus::Connecting => {
                            ui.label(egui::RichText::new("● Connecting...").color(egui::Color32::from_rgb(245, 158, 11)));
                        }
                        HostStatus::Occupied => {
                            ui.label(egui::RichText::new("● Remote Controlled").color(egui::Color32::from_rgb(59, 130, 246)));
                        }
                        HostStatus::Error => {
                            ui.label(egui::RichText::new("● Service Error").color(egui::Color32::from_rgb(239, 68, 68)));
                        }
                    }
                });
            });

            ui.add_space(6.0);
            ui.separator();
            ui.add_space(8.0);

            // Information Card Grid (Simulated mica/glass card)
            egui::Frame::none()
                .fill(egui::Color32::from_rgba_unmultiplied(255, 255, 255, 220))
                .stroke(egui::Stroke::new(1.0, egui::Color32::from_rgb(220, 225, 235)))
                .rounding(10.0)
                .outer_margin(4.0)
                .inner_margin(12.0)
                .show(ui, |ui| {
                    ui.vertical(|ui| {
                        ui.label(egui::RichText::new("Host Information").font(egui::FontId::proportional(14.0)).strong().color(egui::Color32::from_rgb(70, 80, 95)));
                        ui.add_space(6.0);

                        egui::Grid::new("info_grid")
                            .num_columns(2)
                            .spacing([24.0, 8.0])
                            .striped(true)
                            .show(ui, |ui| {
                                ui.label("Device ID:");
                                ui.monospace(&self.device_info.device_id);
                                ui.end_row();

                                ui.label("Computer Name:");
                                ui.label(&self.device_info.device_name);
                                ui.end_row();

                                ui.label("Local IP Address:");
                                ui.label(&self.device_info.local_ip);
                                ui.end_row();

                                ui.label("Listening Port:");
                                ui.monospace(self.device_info.listen_port.to_string());
                                ui.end_row();

                                ui.label("Screen Resolution:");
                                ui.label(format!("{} × {}", self.device_info.screen_width, self.device_info.screen_height));
                                ui.end_row();

                                ui.label("Capturer Module:");
                                ui.label(self.capturer.name());
                                ui.end_row();
                            });
                    });
                });

            ui.add_space(10.0);

            // Active Connection / Control Info
            if self.status == HostStatus::Occupied {
                egui::Frame::none()
                    .fill(egui::Color32::from_rgba_unmultiplied(230, 242, 255, 230))
                    .stroke(egui::Stroke::new(1.0, egui::Color32::from_rgb(180, 210, 255)))
                    .rounding(10.0)
                    .inner_margin(12.0)
                    .show(ui, |ui| {
                        ui.vertical(|ui| {
                            ui.horizontal(|ui| {
                                ui.label(egui::RichText::new("⚠️ Active Remote Session").strong().color(egui::Color32::from_rgb(30, 80, 180)));
                                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                    if ui.button("Disconnect").clicked() {
                                        self.status = HostStatus::Listening;
                                        self.active_session_client_ip = None;
                                        self.log("Remote session closed by host user.");
                                    }
                                });
                            });
                            ui.add_space(4.0);
                            let client_ip = self.active_session_client_ip.clone().unwrap_or_else(|| "192.168.1.150".to_string());
                            ui.label(format!("Controller IP: {}", client_ip));
                            
                            // Metrics
                            let m = self.metrics.lock().unwrap();
                            ui.add_space(4.0);
                            ui.horizontal(|ui| {
                                ui.label(format!("Delay: {:.1} ms", m.latency_ms));
                                ui.label("•");
                                ui.label(format!("FPS: {}", m.fps));
                                ui.label("•");
                                ui.label(format!("Bitrate: {:.0} kbps", m.bitrate_kbps));
                            });
                        });
                    });
            } else {
                // Settings Toggle
                ui.checkbox(&mut self.allow_control, "Allow mouse & keyboard remote control operations");
            }

            ui.add_space(10.0);

            // Action Buttons
            ui.horizontal(|ui| {
                if self.status == HostStatus::Idle {
                    let btn = ui.add(egui::Button::new("Start Listener")
                        .fill(egui::Color32::from_rgb(59, 130, 246))
                        .text_style(egui::TextStyle::Button));
                    if btn.clicked() {
                        self.status = HostStatus::Listening;
                        self.log("Listening socket bind success. Ready for inbound viewers.");
                    }
                } else if self.status == HostStatus::Listening {
                    if ui.button("Stop Listener").clicked() {
                        self.status = HostStatus::Idle;
                        self.log("Listener socket closed.");
                    }

                    // Simulated incoming request button for testing
                    if ui.button("Simulate Inbound Connection Request").clicked() {
                        self.active_request = Some(SessionRequest {
                            session_id: "SESSION-X100".to_string(),
                            viewer_name: "CONTROLLER-LAPTOP".to_string(),
                            viewer_ip: "192.168.1.150".to_string(),
                            request_time: Utc::now(),
                            required_fps: 60,
                            width: 1920,
                            height: 1080,
                        });
                        self.status = HostStatus::Connecting;
                        self.log("Incoming remote request from 192.168.1.150 received.");
                    }
                }

                if ui.button("Refresh IP").clicked() {
                    self.device_info.local_ip = "192.168.1.102".to_string(); // Mock refresh
                    self.log("Refreshed system interfaces. Binding to new IP.");
                }

                if ui.button("Clear Logs").clicked() {
                    self.logs.clear();
                }
            });

            // Pending request modal/popup
            if self.status == HostStatus::Connecting {
                if let Some(req) = &self.active_request {
                    egui::Window::new("Inbound Request")
                        .collapsible(false)
                        .resizable(false)
                        .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
                        .show(ctx, |ui| {
                            ui.label("New connection request received:");
                            ui.add_space(4.0);
                            ui.monospace(format!("Session ID: {}", req.session_id));
                            ui.label(format!("Device: {}", req.viewer_name));
                            ui.label(format!("Source IP: {}", req.viewer_ip));
                            ui.label("Type: Local Area Network (LAN)");
                            ui.add_space(8.0);
                            
                            ui.horizontal(|ui| {
                                if ui.button("Accept Request").clicked() {
                                    self.status = HostStatus::Occupied;
                                    self.active_session_client_ip = Some(req.viewer_ip.clone());
                                    self.active_request = None;
                                    self.session_start_time = Some(Instant::now());
                                    self.log(&format!("Accepted remote session. Controlling allowed: {}", self.allow_control));
                                }
                                if ui.button("Reject").clicked() {
                                    self.status = HostStatus::Listening;
                                    self.active_request = None;
                                    self.log("Rejected connection request.");
                                }
                            });
                        });
                }
            }

            // Logs output window
            ui.add_space(10.0);
            ui.label(egui::RichText::new("Event Logs").strong().color(egui::Color32::from_rgb(110, 115, 125)));
            egui::ScrollArea::vertical()
                .max_height(100.0)
                .stick_to_bottom(true)
                .show(ui, |ui| {
                    ui.vertical(|ui| {
                        for log in &self.logs {
                            ui.monospace(log);
                        }
                    });
                });
        });
    }
}

fn main() -> Result<(), eframe::Error> {
    tracing_subscriber::fmt::init();
    
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([440.0, 520.0])
            .with_resizable(false),
        ..Default::default()
    };
    
    eframe::run_native(
        "ArcLink Host",
        options,
        Box::new(|_cc| Box::new(HostApp::new())),
    )
}
