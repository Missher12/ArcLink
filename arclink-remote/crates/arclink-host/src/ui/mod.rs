use eframe::egui;
use crate::network::{NetworkManager, UserAction};
use crate::logging::HostLogger;
use crate::capture::create_default_capturer;
use arclink_common::HostStatus;

use std::sync::{Arc, Mutex};

pub struct HostUi {
    capturer_name: String,
    screen_width: u32,
    screen_height: u32,
    port_input: String,
    error_message: Option<String>,
}

impl HostUi {
    pub fn new() -> Self {
        let cap = create_default_capturer();
        let info = cap.source_info();
        Self {
            capturer_name: info.name,
            screen_width: info.width,
            screen_height: info.height,
            port_input: "8443".to_string(),
            error_message: None,
        }
    }

    pub fn draw(
        &mut self,
        ctx: &egui::Context,
        net_mgr: &Arc<Mutex<NetworkManager>>,
        logger: &Arc<Mutex<HostLogger>>,
    ) {
        // Core Visual Theme Configuration (Apple Liquid Glass / Mica)
        let mut visuals = egui::Visuals::light();
        visuals.widgets.noninteractive.bg_fill = egui::Color32::from_rgb(245, 247, 250); // BG_BASE
        visuals.widgets.inactive.bg_fill = egui::Color32::from_rgba_unmultiplied(255, 255, 255, 210); // SURFACE_GLASS
        visuals.widgets.hovered.bg_fill = egui::Color32::from_rgba_unmultiplied(241, 247, 255, 235); // SURFACE_HOVER
        visuals.window_rounding = 12.0.into();
        visuals.window_shadow.color = egui::Color32::from_rgba_unmultiplied(100, 110, 130, 20); // soft shadow
        ctx.set_visuals(visuals);

        egui::CentralPanel::default().show(ctx, |ui| {
            let mut net = net_mgr.lock().unwrap();
            let mut log = logger.lock().unwrap();
            let status = net.get_status();

            ui.add_space(10.0);

            // 1. TOP HEADER WITH STATUS CAPSULE
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new("ArcLink Host").font(egui::FontId::proportional(20.0)).strong().color(egui::Color32::from_rgb(36, 48, 64)));
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    draw_status_capsule(ui, status);
                });
            });

            ui.add_space(8.0);
            ui.colored_label(egui::Color32::from_rgb(220, 227, 238), "————————————————————————————————————————————————");
            ui.add_space(8.0);

            // 2. MAIN HOST CONFIGURATION WINDOW (if not occupied)
            if status != HostStatus::Occupied {
                // Large Device Identity Panel
                egui::Frame::none()
                    .fill(egui::Color32::from_rgba_unmultiplied(255, 255, 255, 230))
                    .stroke(egui::Stroke::new(1.0, egui::Color32::from_rgb(220, 227, 238)))
                    .rounding(10.0)
                    .inner_margin(16.0)
                    .show(ui, |ui| {
                        ui.vertical(|ui| {
                            ui.label(egui::RichText::new("WORKSTATION-WIN11-PRO").font(egui::FontId::proportional(16.0)).strong().color(egui::Color32::from_rgb(36, 48, 64)));
                            ui.add_space(2.0);
                            ui.label(egui::RichText::new("被控端安全状态工具").font(egui::FontId::proportional(11.0)).color(egui::Color32::from_rgb(110, 124, 142)));
                            ui.add_space(12.0);

                            // Real IP and Listening port
                            egui::Grid::new("host_identity_grid")
                                .num_columns(2)
                                .spacing([20.0, 10.0])
                                .show(ui, |ui| {
                                    ui.label(egui::RichText::new("局域网 IPv4 地址:").color(egui::Color32::from_rgb(110, 124, 142)));
                                    ui.label(egui::RichText::new(net.local_ip()).strong().color(egui::Color32::from_rgb(58, 126, 235)));
                                    ui.end_row();

                                    ui.label(egui::RichText::new("控制监听端口:").color(egui::Color32::from_rgb(110, 124, 142)));
                                    if status == HostStatus::Listening {
                                        ui.monospace(net.port().to_string());
                                    } else {
                                        ui.horizontal(|ui| {
                                            let res = ui.add(egui::TextEdit::singleline(&mut self.port_input).desired_width(60.0));
                                            if res.changed() {
                                                if let Ok(p) = self.port_input.trim().parse::<u16>() {
                                                    net.set_port(p);
                                                    self.error_message = None;
                                                } else {
                                                    self.error_message = Some("Invalid port number".into());
                                                }
                                            }
                                        });
                                    }
                                    ui.end_row();

                                    ui.label(egui::RichText::new("屏幕采集器:").color(egui::Color32::from_rgb(110, 124, 142)));
                                    ui.label(egui::RichText::new(&self.capturer_name).color(egui::Color32::from_rgb(36, 48, 64)));
                                    ui.end_row();

                                    ui.label(egui::RichText::new("当前分辨率:").color(egui::Color32::from_rgb(110, 124, 142)));
                                    ui.label(format!("{} × {}", self.screen_width, self.screen_height));
                                    ui.end_row();
                                });
                        });
                    });

                if let Some(ref err) = self.error_message {
                    ui.add_space(6.0);
                    ui.colored_label(egui::Color32::from_rgb(213, 85, 86), err);
                }

                ui.add_space(16.0);

                // Control Action Button for Listener
                ui.horizontal(|ui| {
                    if status == HostStatus::Idle || status == HostStatus::Error {
                        let btn = ui.add(egui::Button::new("  开启局域网监听  ")
                            .fill(egui::Color32::from_rgb(58, 126, 235)) // ACCENT_BLUE
                            .rounding(6.0));
                        if btn.clicked() {
                            if let Err(e) = net.start_listening(logger.clone()) {
                                self.error_message = Some(e);
                            } else {
                                self.error_message = None;
                            }
                        }
                    } else if status == HostStatus::Listening {
                        let btn = ui.add(egui::Button::new("  关闭服务监听  ")
                            .fill(egui::Color32::from_rgb(213, 85, 86)) // STATUS_RED
                            .rounding(6.0));
                        if btn.clicked() {
                            net.stop_listening(logger.clone());
                        }
                    }
                });
            }

            // 3. REMOTE CONTROLLED (OCCUPIED) GLASS STATE CARD
            if status == HostStatus::Occupied {
                if let Some(sess) = net.get_active_session() {
                    egui::Frame::none()
                        .fill(egui::Color32::from_rgba_unmultiplied(240, 245, 255, 240)) // Light blue translucent glass card
                        .stroke(egui::Stroke::new(1.0, egui::Color32::from_rgb(180, 205, 245)))
                        .rounding(12.0)
                        .inner_margin(18.0)
                        .show(ui, |ui| {
                            ui.vertical(|ui| {
                                ui.label(egui::RichText::new("正在被远程控制中").font(egui::FontId::proportional(15.0)).strong().color(egui::Color32::from_rgb(58, 126, 235)));
                                ui.add_space(6.0);
                                
                                ui.label(format!("控制端 ID: {}", sess.viewer_id));
                                ui.label(format!("控制端 IP: {}", sess.viewer_ip));
                                ui.label(format!("连接协议: LAN Direct (v{})", sess.protocol_version));
                                ui.add_space(10.0);

                                // Real-time session metrics
                                let metrics = net.metrics_mgr.get_metrics();
                                ui.horizontal(|ui| {
                                    if let Some(rtt) = metrics.control_rtt_ms {
                                        ui.label(format!("网络延迟: {:.1} ms", rtt));
                                        ui.label("•");
                                    }
                                    if let Some(fps) = metrics.capture_fps {
                                        ui.label(format!("采集率: {:.0} FPS", fps));
                                        ui.label("•");
                                    }
                                    ui.label(format!("时长: {}s", metrics.active_duration_secs));
                                });

                                ui.add_space(14.0);
                                if ui.add(egui::Button::new(" 立即断开控制会话 ")
                                    .fill(egui::Color32::from_rgb(213, 85, 86))
                                    .rounding(6.0)).clicked() {
                                        net.trigger_user_action(UserAction::Disconnect);
                                }
                            });
                        });
                }
            }

            // 4. HANDSHAKE INBOUND CONFIRMATION WINDOW
            if status == HostStatus::Connecting {
                if let Some(req) = net.get_active_request() {
                    // Draw a gorgeous frosted glass overlay modal
                    egui::Window::new(" 局域网接入申请 confirmation ")
                        .collapsible(false)
                        .resizable(false)
                        .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
                        .show(ctx, |ui| {
                            ui.set_max_width(280.0);
                            ui.vertical(|ui| {
                                ui.label(egui::RichText::new("检测到远程客户端请求接入").strong().color(egui::Color32::from_rgb(36, 48, 64)));
                                ui.add_space(6.0);

                                egui::Grid::new("confirm_modal_grid").num_columns(2).spacing([12.0, 8.0]).show(ui, |ui| {
                                    ui.label("主控端 ID:");
                                    ui.monospace(&req.viewer_name);
                                    ui.end_row();

                                    ui.label("局域网 IP:");
                                    ui.label(&req.viewer_ip);
                                    ui.end_row();

                                    ui.label("连接模式:");
                                    ui.colored_label(egui::Color32::from_rgb(58, 126, 235), "LAN Direct");
                                    ui.end_row();
                                });

                                ui.add_space(14.0);
                                ui.horizontal(|ui| {
                                    // Accept (clipt blue)
                                    let acc_btn = ui.add_sized([100.0, 28.0], egui::Button::new(" 接受连接 ")
                                        .fill(egui::Color32::from_rgb(58, 126, 235)) // ACCENT_BLUE
                                        .rounding(6.0));
                                    if acc_btn.clicked() {
                                        net.trigger_user_action(UserAction::Accept);
                                    }

                                    ui.add_space(10.0);

                                    // Reject (outline soft red)
                                    let rej_btn = ui.add_sized([100.0, 28.0], egui::Button::new(" 拒绝 ")
                                        .fill(egui::Color32::from_rgb(240, 240, 240))
                                        .stroke(egui::Stroke::new(1.0, egui::Color32::from_rgb(213, 85, 86)))
                                        .rounding(6.0));
                                    if rej_btn.clicked() {
                                        net.trigger_user_action(UserAction::Reject);
                                    }
                                });
                            });
                        });
                }
            }

            // 5. EVENT LOGS RING BUFFER
            ui.add_space(14.0);
            ui.label(egui::RichText::new("事件日志 (Event Telemetry)").strong().color(egui::Color32::from_rgb(110, 124, 142)));
            egui::ScrollArea::vertical()
                .max_height(100.0)
                .stick_to_bottom(true)
                .show(ui, |ui| {
                    ui.vertical(|ui| {
                        for log_line in log.get_ui_logs() {
                            ui.monospace(log_line);
                        }
                    });
                });

            ui.add_space(8.0);
            ui.horizontal(|ui| {
                if ui.small_button("清除日志").clicked() {
                    log.clear();
                }
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(egui::RichText::new(format!("Log: {}", log.file_path_str())).font(egui::FontId::monospace(9.0)).color(egui::Color32::from_rgb(160, 170, 185)));
                });
            });
        });
    }
}

fn draw_status_capsule(ui: &mut egui::Ui, status: HostStatus) {
    let (text, color_dot, color_bg) = match status {
        HostStatus::Idle => ("未启动 (Offline)", egui::Color32::from_rgb(150, 150, 150), egui::Color32::from_rgb(230, 230, 230)),
        HostStatus::Listening => ("准备就绪 (Listening)", egui::Color32::from_rgb(52, 168, 108), egui::Color32::from_rgb(235, 247, 241)),
        HostStatus::Connecting => ("握手接入中...", egui::Color32::from_rgb(222, 151, 57), egui::Color32::from_rgb(254, 245, 231)),
        HostStatus::Occupied => ("远程主控连接中 (Active)", egui::Color32::from_rgb(58, 126, 235), egui::Color32::from_rgb(235, 242, 254)),
        HostStatus::Error => ("端口被占用 (Error)", egui::Color32::from_rgb(213, 85, 86), egui::Color32::from_rgb(254, 238, 238)),
    };

    egui::Frame::none()
        .fill(color_bg)
        .rounding(12.0)
        .inner_margin(egui::vec2(8.0, 4.0))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.circle_filled(egui::pos2(ui.cursor().min.x + 4.0, ui.cursor().center().y), 3.0, color_dot);
                ui.add_space(8.0);
                ui.label(egui::RichText::new(text).font(egui::FontId::proportional(11.0)).strong().color(color_dot));
            });
        });
}
