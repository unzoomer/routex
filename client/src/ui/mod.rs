use eframe::egui;
use egui::{Color32, FontId, RichText, Stroke, Vec2};

#[derive(Default)]
pub struct RouteXApp {
    connected: bool,
    selected_server: usize,
    ping_history: Vec<f32>,
    frame: u64,
}

impl RouteXApp {
    fn servers() -> Vec<(&'static str, &'static str, f32)> {
        vec![
            ("🇩🇪 FRANKFURT-01", "eu-central · wg0", 24.0),
            ("🇵🇱 WARSAW-03",    "eu-east · wg0",    31.0),
            ("🇳🇱 AMSTERDAM-02", "eu-west · wg0",    41.0),
        ]
    }
}

const BG:      Color32 = Color32::from_rgb(12,  12,  15);
const BG2:     Color32 = Color32::from_rgb(17,  17,  24);
const CYAN:    Color32 = Color32::from_rgb(0,   240, 255);
const CYAN_DIM:Color32 = Color32::from_rgb(0,   80,  90);
const RED:     Color32 = Color32::from_rgb(255, 0,   60);
const GRAY:    Color32 = Color32::from_rgb(40,  60,  70);
const TEXT:    Color32 = Color32::from_rgb(200, 216, 232);

impl eframe::App for RouteXApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.frame += 1;

        // Симуляция пинга
        if self.connected && self.frame % 30 == 0 {
            let ping = 20.0 + (self.frame as f32 * 0.3).sin().abs() * 10.0;
            self.ping_history.push(ping);
            if self.ping_history.len() > 40 {
                self.ping_history.remove(0);
            }
        }

        let style = ctx.style_mut(|s| {
            s.visuals.window_fill = BG;
            s.visuals.panel_fill  = BG;
            s.visuals.override_text_color = Some(TEXT);
        });

        // Сайдбар
        egui::SidePanel::left("sidebar")
            .exact_width(160.0)
            .frame(egui::Frame::none().fill(BG2).stroke(Stroke::new(1.0, GRAY)))
            .show(ctx, |ui| {
                ui.add_space(12.0);

                ui.label(RichText::new("ROUTEX")
                    .font(FontId::monospace(18.0))
                    .color(CYAN)
                    .strong());

                ui.label(RichText::new("v0.1.0-alpha")
                    .font(FontId::monospace(10.0))
                    .color(CYAN_DIM));

                ui.add_space(16.0);
                ui.separator();
                ui.add_space(8.0);

                for (label, active) in [
                    ("> /dashboard", true),
                    ("> /nodes",     false),
                    ("> /games",     false),
                    ("> /logs",      false),
                    ("> /config",    false),
                ] {
                    let color = if active { CYAN } else { GRAY };
                    ui.label(RichText::new(label)
                        .font(FontId::monospace(11.0))
                        .color(color));
                    ui.add_space(4.0);
                }
            });

        // Главная панель
        egui::CentralPanel::default()
            .frame(egui::Frame::none().fill(BG).inner_margin(14.0))
            .show(ctx, |ui| {

                // Статус
                let status_color = if self.connected { CYAN } else { RED };
                let status_text  = if self.connected {
                    "● CONNECTED → FRANKFURT-01"
                } else {
                    "○ DISCONNECTED → IDLE"
                };

                ui.horizontal(|ui| {
                    ui.label(RichText::new(status_text)
                        .font(FontId::monospace(12.0))
                        .color(status_color));

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        let btn_text  = if self.connected { "[ DISCONNECT ]" } else { "[ CONNECT ]" };
                        let btn_color = if self.connected { RED } else { CYAN };
                        if ui.button(RichText::new(btn_text)
                            .font(FontId::monospace(11.0))
                            .color(btn_color)).clicked()
                        {
                            self.connected = !self.connected;
                            if self.connected { self.ping_history.clear(); }
                        }
                    });
                });

                ui.add_space(10.0);

                // Метрики
                let ping_val = self.ping_history.last().copied().unwrap_or(0.0);
                ui.horizontal(|ui| {
                    for (label, value, unit) in [
                        ("LATENCY", format!("{:.0}", ping_val), "ms"),
                        ("PKT LOSS", "0.2".to_string(), "%"),
                        ("TRAFFIC",  "1.4".to_string(), "mb/s"),
                    ] {
                        egui::Frame::none()
                            .fill(BG2)
                            .stroke(Stroke::new(1.0, GRAY))
                            .inner_margin(8.0)
                            .show(ui, |ui| {
                                ui.set_min_width(100.0);
                                ui.label(RichText::new(label)
                                    .font(FontId::monospace(9.0))
                                    .color(CYAN_DIM));
                                ui.label(RichText::new(format!("{} {}", value, unit))
                                    .font(FontId::monospace(18.0))
                                    .color(CYAN));
                            });
                        ui.add_space(6.0);
                    }
                });

                ui.add_space(10.0);

                // График пинга
                ui.label(RichText::new("# ping_monitor --live")
                    .font(FontId::monospace(10.0))
                    .color(CYAN_DIM));
                ui.add_space(4.0);

                let plot_size = Vec2::new(ui.available_width(), 60.0);
                let (rect, _) = ui.allocate_exact_size(plot_size, egui::Sense::hover());
                let painter = ui.painter();
                painter.rect_filled(rect, 2.0, BG2);

                if self.ping_history.len() > 1 {
                    let n = self.ping_history.len();
                    let points: Vec<egui::Pos2> = self.ping_history.iter().enumerate().map(|(i, &v)| {
                        let x = rect.left() + (i as f32 / (n - 1) as f32) * rect.width();
                        let y = rect.bottom() - ((v - 10.0) / 40.0).clamp(0.0, 1.0) * rect.height();
                        egui::Pos2::new(x, y)
                    }).collect();

                    for w in points.windows(2) {
                        painter.line_segment([w[0], w[1]], Stroke::new(1.5, CYAN));
                    }
                }

                ui.add_space(10.0);

                // Серверы
                ui.label(RichText::new("# node_list --sort=latency")
                    .font(FontId::monospace(10.0))
                    .color(CYAN_DIM));
                ui.add_space(4.0);

                for (i, (name, sub, ping)) in Self::servers().iter().enumerate() {
                    let is_sel = self.selected_server == i;
                    let frame_color = if is_sel { CYAN } else { GRAY };

                    egui::Frame::none()
                        .fill(BG2)
                        .stroke(Stroke::new(if is_sel { 1.5 } else { 1.0 }, frame_color))
                        .inner_margin(8.0)
                        .show(ui, |ui| {
                            ui.horizontal(|ui| {
                                ui.label(RichText::new(if is_sel { "[*]" } else { "[ ]" })
                                    .font(FontId::monospace(11.0))
                                    .color(frame_color));
                                ui.label(RichText::new(*name)
                                    .font(FontId::monospace(11.0))
                                    .color(TEXT));
                                ui.label(RichText::new(*sub)
                                    .font(FontId::monospace(9.0))
                                    .color(GRAY));
                                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                    let pc = if *ping < 35.0 { CYAN } else { Color32::from_rgb(170,255,0) };
                                    ui.label(RichText::new(format!("{:.0}ms", ping))
                                        .font(FontId::monospace(11.0))
                                        .color(pc));
                                });
                            });
                        });

                    if ui.interact(ui.min_rect(), ui.id().with(i), egui::Sense::click()).clicked() {
                        self.selected_server = i;
                    }
                    ui.add_space(4.0);
                }
            });

        ctx.request_repaint();
    }
}