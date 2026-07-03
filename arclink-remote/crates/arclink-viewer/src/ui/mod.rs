use eframe::egui;
use crate::network::NetworkManager;
use crate::logging::ViewerLogger;
use crate::render::VideoFrameBuffer;
use crate::input::map_key_to_vk;
use arclink_common::{InputEvent, MouseMoveEvent, MouseButtonEvent, MouseButton, MouseWheelEvent, KeyboardEvent, ViewerStatus};

use std::sync::{Arc, Mutex};

pub struct ViewerUi {
    host_ip_input: String,
    port_input: String,
    show_metrics_panel: bool,
    texture_id: Option<egui::TextureHandle>,
    last_mouse_pos: Option<egui::Pos2>,
}

impl ViewerUi {
    pub fn new() -> Self {
        Self {
            host_ip_input: "127.0.0.1".to_string(),
            port_input: "8443".to_string(),
            show_metrics_panel: true,
            texture_id: None,
            last_mouse_pos: None,
        }
    }

    pub fn draw(
        &mut self,
        ctx: &egui::Context,
        net_mgr: &Arc<Mutex<NetworkManager>>,
        logger: &Arc<Mutex<ViewerLogger>>,
        frame_buffer: &Arc<Mutex<VideoFrameBuffer>>,
    ) {
        // Aesthetic setup matching Apple Liquid Glass theme
        let mut visuals = egui::Visuals::light();
        visuals.widgets.noninteractive.bg_fill = egui::Color32::from_rgb(245, 247, 250); // BG_BASE
        visuals.widgets.inactive.bg_fill = egui::Color32::from_rgba_unmultiplied(255, 255, 255, 210); // GLASS
        visuals.window_rounding = 12.0.into();
        visuals.window_shadow.color = egui::Color32::from_rgba_unmultiplied(100, 110, 130, 20);
        ctx.set_visuals(visuals);

        let status = net_mgr.lock().unwrap().get_status();

        if status == ViewerStatus::Connected {
            // Draw immersive full screen remote desktop with floating top overlay
            egui::CentralPanel::default()
                .frame(egui::Frame::none().fill(egui::Color32::from_rgb(20, 22, 26))) // dark canvas background
                .show(ctx, |ui| {
                    self.draw_remote_desktop(ui, ctx, net_mgr, frame_buffer);
                    self.draw_floating_control_bar(ui, ctx, net_mgr, logger);
                });
        } else {
            // Disconnected / Connection screen
            egui::CentralPanel::default().show(ctx, |ui| {
                ui.add_space(20.0);

                // Title
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new("ArcLink Viewer").font(egui::FontId::proportional(22.0)).strong().color(egui::Color32::from_rgb(36, 48, 64)));
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        draw_status_capsule(ui, status);
                    });
                });

                ui.add_space(8.0);
                ui.colored_label(egui::Color32::from_rgb(220, 227, 238), "————————————————————————————————————————————————");
                ui.add_space(14.0);

                // Card
                egui::Frame::none()
                    .fill(egui::Color32::from_rgba_unmultiplied(255, 255, 255, 230))
                    .stroke(egui::Stroke::new(1.0, egui::Color32::from_rgb(220, 227, 238)))
                    .rounding(10.0)
                    .inner_margin(18.0)
                    .show(ui, |ui| {
                        ui.vertical(|ui| {
                            ui.label(egui::RichText::new("建立新的远程控制会话").font(egui::FontId::proportional(15.0)).strong().color(egui::Color32::from_rgb(36, 48, 64)));
                            ui.add_space(4.0);
                            ui.label(egui::RichText::new("请输入被控端的局域网 IPv4 地址和控制端口进行直连：").font(egui::FontId::proportional(11.0)).color(egui::Color32::from_rgb(110, 124, 142)));
                            ui.add_space(14.0);

                            egui::Grid::new("connect_form_grid")
                                .num_columns(2)
                                .spacing([20.0, 12.0])
                                .show(ui, |ui| {
                                    ui.label(egui::RichText::new("被控端 IP:").color(egui::Color32::from_rgb(36, 48, 64)));
                                    ui.text_edit_singleline(&mut self.host_ip_input);
                                    ui.end_row();

                                    ui.label(egui::RichText::new("控制监听端口:").color(egui::Color32::from_rgb(36, 48, 64)));
                                    ui.text_edit_singleline(&mut self.port_input);
                                    ui.end_row();
                                });
                        });
                    });

                ui.add_space(16.0);

                // Action Button
                if status == ViewerStatus::Connecting {
                    ui.add_enabled(false, egui::Button::new(" 正在连接并等待确认... ")
                        .fill(egui::Color32::from_rgb(58, 126, 235))
                        .rounding(6.0));
                } else {
                    let conn_btn = ui.add(egui::Button::new("  开启远程控制会话  ")
                        .fill(egui::Color32::from_rgb(58, 126, 235)) // ACCENT_BLUE
                        .rounding(6.0));
                    if conn_btn.clicked() {
                        let port_parsed = self.port_input.trim().parse::<u16>().unwrap_or(8443);
                        let mut net = net_mgr.lock().unwrap();
                        let _ = net.connect_to_host(self.host_ip_input.trim().to_string(), port_parsed, logger.clone());
                    }
                }

                // Logs Terminal (for diagnostics)
                ui.add_space(20.0);
                ui.label(egui::RichText::new("本地网络调试日志 (Viewer Logs)").strong().color(egui::Color32::from_rgb(110, 124, 142)));
                
                let mut log = logger.lock().unwrap();
                egui::ScrollArea::vertical()
                    .max_height(110.0)
                    .stick_to_bottom(true)
                    .show(ui, |ui| {
                        ui.vertical(|ui| {
                            for line in log.get_ui_logs() {
                                ui.monospace(line);
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

    fn draw_remote_desktop(
        &mut self,
        ui: &mut egui::Ui,
        ctx: &egui::Context,
        net_mgr: &Arc<Mutex<NetworkManager>>,
        frame_buffer: &Arc<Mutex<VideoFrameBuffer>>,
    ) {
        let frame_opt = frame_buffer.lock().unwrap().take_latest_frame();

        if let Some(decoded) = frame_opt {
            // Convert Decoded RGBA frame to egui texture handle
            let size = [decoded.width as usize, decoded.height as usize];
            let color_image = egui::ColorImage::from_rgba_unmultiplied(size, &decoded.rgba_bytes);
            
            // Reallocate or update existing texture
            let texture = ctx.load_texture("remote_desktop", color_image, Default::default());
            self.texture_id = Some(texture.clone());

            // Render texture taking full screen layout space
            let canvas_rect = ui.max_rect();
            ui.put(canvas_rect, egui::Image::new(&texture).fit_to_exact_size(canvas_rect.size()));

            // -------------------------------------------------------------
            // INPUT EVENT CAPTURE ON CANVAS
            // -------------------------------------------------------------
            let canvas_response = ui.interact(canvas_rect, ui.id(), egui::Sense::click_and_drag());
            
            // Only send mouse events if hovering the canvas
            if let Some(hover_pos) = canvas_response.hover_pos() {
                let norm_x = (hover_pos.x - canvas_rect.min.x) / canvas_rect.width();
                let norm_y = (hover_pos.y - canvas_rect.min.y) / canvas_rect.height();

                // Mouse movement throttle / limit to prevent packet flood
                let cur_pos = hover_pos;
                let should_send_move = match self.last_mouse_pos {
                    Some(last) => cur_pos.distance_sq(last) > 4.0, // send on change > 2 pixels
                    None => true,
                };

                if should_send_move && norm_x >= 0.0 && norm_x <= 1.0 && norm_y >= 0.0 && norm_y <= 1.0 {
                    net_mgr.lock().unwrap().send_input_event(InputEvent::MouseMove(MouseMoveEvent {
                        norm_x,
                        norm_y,
                    }));
                    self.last_mouse_pos = Some(cur_pos);
                }

                // Handle clicks
                if canvas_response.clicked_by(egui::PointerButton::Primary) {
                    send_click(net_mgr, MouseButton::Left, true);
                    send_click(net_mgr, MouseButton::Left, false);
                } else if canvas_response.secondary_clicked() {
                    send_click(net_mgr, MouseButton::Right, true);
                    send_click(net_mgr, MouseButton::Right, false);
                }
            }

            // Capture Keyboard keys when canvas is active & focused
            if canvas_response.has_focus() || canvas_response.clicked() {
                ui.input(|i| {
                    for event in &i.events {
                        match event {
                            egui::Event::Key { key, pressed, modifiers, .. } => {
                                let vk = map_key_to_vk(*key);
                                if vk != 0 {
                                    let mut mod_flags = 0u8;
                                    if modifiers.shift { mod_flags |= 1; }
                                    if modifiers.ctrl { mod_flags |= 2; }
                                    if modifiers.alt { mod_flags |= 4; }
                                    if modifiers.command { mod_flags |= 8; }

                                    net_mgr.lock().unwrap().send_input_event(InputEvent::Keyboard(KeyboardEvent {
                                        vk_code: vk,
                                        is_down: *pressed,
                                        modifiers: mod_flags,
                                    }));
                                }
                            }
                            _ => {}
                        }
                    }
                });
            }
        } else {
            // Draw empty waiting screen
            ui.centered_and_justified(|ui| {
                ui.vertical_centered(|ui| {
                    ui.label(egui::RichText::new("正在等待远程画面载入...").font(egui::FontId::proportional(15.0)).strong().color(egui::Color32::from_rgb(110, 124, 142)));
                    ui.add_space(4.0);
                    ui.label(egui::RichText::new("已建立控制信道，等待视频通道流拼包重组...").font(egui::FontId::proportional(11.0)).color(egui::Color32::from_rgb(80, 95, 110)));
                });
            });
        }
    }

    fn draw_floating_control_bar(
        &mut self,
        ui: &mut egui::Ui,
        ctx: &egui::Context,
        net_mgr: &Arc<Mutex<NetworkManager>>,
        logger: &Arc<Mutex<ViewerLogger>>,
    ) {
        // Render a floating control panel at the top center of the remote desktop
        let status = net_mgr.lock().unwrap().get_status();
        let active_sess_opt = net_mgr.lock().unwrap().get_active_session();

        if let Some(sess) = active_sess_opt {
            let screen_width = ui.max_rect().width();
            let bar_width = 440.0;
            let bar_rect = egui::Rect::from_min_size(
                egui::pos2((screen_width - bar_width) / 2.0, 10.0),
                egui::vec2(bar_width, 42.0)
            );

            // Floating Frosted Glass panel
            ui.put(bar_rect, egui::Frame::none()
                .fill(egui::Color32::from_rgba_unmultiplied(255, 255, 255, 210)) // liquid glass overlay
                .stroke(egui::Stroke::new(1.0, egui::Color32::from_rgb(220, 227, 238)))
                .rounding(10.0)
                .inner_margin(egui::vec2(12.0, 6.0))
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.label(egui::RichText::new("ArcLink Viewer").strong().color(egui::Color32::from_rgb(36, 48, 64)));
                        ui.separator();

                        ui.label(format!("受控 IP: {}", sess.host_ip));
                        ui.separator();

                        // Collapsible stats trigger
                        ui.checkbox(&mut self.show_metrics_panel, "实时性能指标");

                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if ui.add(egui::Button::new(" 结束控制 ")
                                .fill(egui::Color32::from_rgb(213, 85, 86))
                                .rounding(4.0)).clicked() {
                                    net_mgr.lock().unwrap().disconnect_from_host();
                            }
                        });
                    });
                }).response);

            // If metrics check is active, render collapsible floating panel
            if self.show_metrics_panel {
                let m_rect = egui::Rect::from_min_size(
                    egui::pos2((screen_width - bar_width) / 2.0, 58.0),
                    egui::vec2(bar_width, 54.0)
                );

                ui.put(m_rect, egui::Frame::none()
                    .fill(egui::Color32::from_rgba_unmultiplied(255, 255, 255, 195))
                    .stroke(egui::Stroke::new(1.0, egui::Color32::from_rgb(220, 227, 238)))
                    .rounding(10.0)
                    .inner_margin(egui::vec2(12.0, 8.0))
                    .show(ui, |ui| {
                        let metrics = net_mgr.lock().unwrap().metrics_mgr.get_metrics();
                        egui::Grid::new("viewer_metrics_grid")
                            .num_columns(4)
                            .spacing([24.0, 4.0])
                            .show(ui, |ui| {
                                ui.label("网络延时:");
                                if let Some(rtt) = metrics.control_rtt_ms {
                                    ui.colored_label(egui::Color32::from_rgb(58, 126, 235), format!("{:.1} ms", rtt));
                                } else {
                                    ui.label("—");
                                }

                                ui.label("解码帧率:");
                                if let Some(fps) = metrics.render_fps {
                                    ui.colored_label(egui::Color32::from_rgb(52, 168, 108), format!("{:.0} FPS", fps));
                                } else {
                                    ui.label("—");
                                }
                                ui.end_row();

                                ui.label("流码率:");
                                if let Some(kbps) = metrics.bitrate_kbps {
                                    ui.label(format!("{:.0} kbps", kbps));
                                } else {
                                    ui.label("—");
                                }

                                ui.label("会话时长:");
                                ui.label(format!("{} 秒", metrics.active_duration_secs));
                                ui.end_row();
                            });
                    }).response);
            }
        }
    }
}

fn send_click(net_mgr: &Arc<Mutex<NetworkManager>>, button: MouseButton, is_down: bool) {
    net_mgr.lock().unwrap().send_input_event(InputEvent::MouseButton(MouseButtonEvent {
        button,
        is_down,
    }));
}

fn draw_status_capsule(ui: &mut egui::Ui, status: ViewerStatus) {
    let (text, color_dot, color_bg) = match status {
        ViewerStatus::Disconnected => ("离线 (Disconnected)", egui::Color32::from_rgb(150, 150, 150), egui::Color32::from_rgb(230, 230, 230)),
        ViewerStatus::Connecting => ("正在握手直连...", egui::Color32::from_rgb(222, 151, 57), egui::Color32::from_rgb(254, 245, 231)),
        ViewerStatus::Connected => ("连接正常 (Live Connection)", egui::Color32::from_rgb(52, 168, 108), egui::Color32::from_rgb(235, 247, 241)),
        ViewerStatus::Reconnecting => ("重新连接中...", egui::Color32::from_rgb(222, 151, 57), egui::Color32::from_rgb(254, 245, 231)),
        ViewerStatus::Error => ("直连握手拒绝 (Refused)", egui::Color32::from_rgb(213, 85, 86), egui::Color32::from_rgb(254, 238, 238)),
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
