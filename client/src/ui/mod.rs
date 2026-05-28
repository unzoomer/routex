use eframe::egui;
use egui::{Color32, FontId, RichText, Stroke, Vec2};

pub struct RouteXApp {
    connected: bool,
    selected_server: usize,
    ping_history: Vec<f32>,
    frame: u64,
    tunnel_tx: Option<std::sync::mpsc::Sender<bool>>,
    ping_rx: Option<std::sync::mpsc::Receiver<f32>>,
    current_ping: f32,
}

impl Default for RouteXApp {
    fn default() -> Self {
        Self {
            connected: false,
            selected_server: 0,
            ping_history: vec![38.0,35.0,30.0,28.0,26.0,24.0,27.0,25.0,23.0,24.0],
            frame: 0,
            tunnel_tx: None,
            ping_rx: None,
            current_ping: 0.0,
        }
    }
}

const BG:       Color32 = Color32::from_rgb(12,  12,  15);
const BG2:      Color32 = Color32::from_rgb(17,  17,  24);
const CYAN:     Color32 = Color32::from_rgb(0,   240, 255);
const CYAN_DIM: Color32 = Color32::from_rgb(0,   80,  90);
const RED:      Color32 = Color32::from_rgb(255, 0,   60);
const GRAY:     Color32 = Color32::from_rgb(40,  60,  70);
const TEXT:     Color32 = Color32::from_rgb(200, 216, 232);
const YELLOW:   Color32 = Color32::from_rgb(170, 255, 0);

struct Server {
    name: &'static str,
    sub:  &'static str,
    ping: f32,
}

const SERVERS: &[Server] = &[
    Server { name: "FRANKFURT-01", sub: "eu-central · wg0", ping: 24.0 },
    Server { name: "WARSAW-03",    sub: "eu-east · wg0",    ping: 31.0 },
    Server { name: "AMSTERDAM-02", sub: "eu-west · wg0",    ping: 41.0 },
];

impl eframe::App for RouteXApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.frame += 1;

        // Реальный пинг из фонового потока
        if let Some(rx) = &self.ping_rx {
            if let Ok(ping) = rx.try_recv() {
                self.current_ping = ping;
                self.ping_history.push(ping);
                if self.ping_history.len() > 50 {
                    self.ping_history.remove(0);
                }
            }
        }

        ctx.style_mut(|s| {
            s.visuals.panel_fill = BG;
            s.visuals.window_fill = BG;
            s.visuals.override_text_color = Some(TEXT);
            s.visuals.widgets.inactive.bg_fill = BG2;
            s.visuals.widgets.hovered.bg_fill = Color32::from_rgb(20, 30, 35);
            s.visuals.widgets.active.bg_fill = Color32::from_rgb(0, 40, 50);
        });

        // Сайдбар
        egui::SidePanel::left("sidebar")
            .exact_width(155.0)
            .frame(egui::Frame::none()
                .fill(BG2)
                .stroke(Stroke::new(1.0, GRAY)))
            .show(ctx, |ui| {
                ui.add_space(14.0);
                ui.horizontal(|ui| {
                    ui.add_space(12.0);
                    ui.label(RichText::new("ROUTE")
                        .font(FontId::monospace(16.0))
                        .color(CYAN)
                        .strong());
                    ui.label(RichText::new("X")
                        .font(FontId::monospace(16.0))
                        .color(RED)
                        .strong());
                });
                ui.add_space(2.0);
                ui.horizontal(|ui| {
                    ui.add_space(12.0);
                    ui.label(RichText::new("v0.1.0-alpha")
                        .font(FontId::monospace(9.0))
                        .color(CYAN_DIM));
                });
                ui.add_space(14.0);

                for (name, active) in [
                    ("dashboard", true),
                    ("nodes",     false),
                    ("games",     false),
                    ("logs",      false),
                    ("config",    false),
                    ("identity",  false),
                ] {
                    let color = if active { CYAN } else { GRAY };
                    let prefix = if active { "> " } else { "  " };
                    ui.horizontal(|ui| {
                        ui.add_space(10.0);
                        ui.label(RichText::new(format!("{}/{}", prefix, name))
                            .font(FontId::monospace(11.0))
                            .color(color));
                    });
                    ui.add_space(3.0);
                }

                ui.add_space(20.0);
                ui.horizontal(|ui| {
                    ui.add_space(10.0);
                    ui.label(RichText::new("────────────────")
                        .font(FontId::monospace(9.0))
                        .color(GRAY));
                });
                ui.add_space(6.0);

                let uptime = self.frame / 60;
                let mins = uptime / 60;
                let secs = uptime % 60;
                ui.horizontal(|ui| {
                    ui.add_space(10.0);
                    ui.label(RichText::new(format!("UPTIME {:02}:{:02}", mins, secs))
                        .font(FontId::monospace(9.0))
                        .color(CYAN_DIM));
                });
                ui.horizontal(|ui| {
                    ui.add_space(10.0);
                    ui.label(RichText::new("PROTO  wireguard")
                        .font(FontId::monospace(9.0))
                        .color(CYAN_DIM));
                });
            });

        // Главная панель
        egui::CentralPanel::default()
            .frame(egui::Frame::none()
                .fill(BG)
                .inner_margin(egui::Margin::symmetric(14.0, 12.0)))
            .show(ctx, |ui| {

                // Статус + кнопка
                ui.horizontal(|ui| {
                    let (dot, status_color, status_text) = if self.connected {
                        ("●", CYAN, format!("CONNECTED → {}", SERVERS[self.selected_server].name))
                    } else {
                        ("○", RED, "DISCONNECTED → IDLE".to_string())
                    };

                    ui.label(RichText::new(dot)
                        .font(FontId::monospace(12.0))
                        .color(status_color));
                    ui.label(RichText::new(&status_text)
                        .font(FontId::monospace(11.0))
                        .color(status_color));

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        let (btn_text, btn_color) = if self.connected {
                            ("[ DISCONNECT ]", RED)
                        } else {
                            ("[ CONNECT ]", CYAN)
                        };
                        let btn = ui.add(egui::Button::new(
                            RichText::new(btn_text)
                                .font(FontId::monospace(11.0))
                                .color(btn_color))
                            .fill(BG2)
                            .stroke(Stroke::new(1.0, btn_color)));

                        if btn.clicked() {
                            self.connected = !self.connected;
                            if self.connected {
                                // Туннель
                                let (tx, rx) = std::sync::mpsc::channel::<bool>();
                                self.tunnel_tx = Some(tx);

                                std::thread::spawn(move || {
                                    let rt = tokio::runtime::Runtime::new().unwrap();
                                    rt.block_on(async {
                                        match crate::tunnel::create_adapter() {
                                            Ok(adapter) => {
                                                let tunnel = crate::tunnel::WireGuardTunnel::new(
                                                    adapter,
                                                    "139.100.219.5:51820",
                                                    &std::env::var("ROUTEX_PRIVATE_KEY").unwrap_or_default(),
                                                    "s8qNGa7xgugqUQSpLEgiLRo6yrNRcAZFc3zPn5zQMmw=",
                                                );
                                                if let Err(e) = tunnel.run().await {
                                                    log::error!("Tunnel error: {}", e);
                                                }
                                            }
                                            Err(e) => log::error!("Adapter error: {}", e),
                                        }
                                        let _ = rx;
                                    });
                                });

                                // Пинг в фоне
                                let (ping_tx, ping_rx) = std::sync::mpsc::channel::<f32>();
                                self.ping_rx = Some(ping_rx);

                                std::thread::spawn(move || {
                                    loop {
                                        let start = std::time::Instant::now();
                                        let sock = std::net::UdpSocket::bind("0.0.0.0:0").unwrap();
                                        sock.set_read_timeout(Some(
                                            std::time::Duration::from_secs(2)
                                        )).unwrap();
                                        let _ = sock.send_to(b"ping", "139.100.219.5:7777");
                                        let ping = start.elapsed().as_millis() as f32;
                                        let _ = ping_tx.send(ping);
                                        std::thread::sleep(std::time::Duration::from_secs(1));
                                    }
                                });

                            } else {
                                self.tunnel_tx = None;
                                self.ping_rx = None;
                                self.current_ping = 0.0;
                            }
                        }
                    });
                });

                ui.add_space(10.0);

                // Метрики
                ui.horizontal(|ui| {
                    let metrics = [
                        ("LATENCY", format!("{:.0}ms", if self.connected { self.current_ping } else { 0.0 }), CYAN),
                        ("PKT LOSS", "0.2%".to_string(), CYAN),
                        ("TRAFFIC",  "1.4 mb/s".to_string(), YELLOW),
                    ];
                    for (label, value, color) in metrics {
                        egui::Frame::none()
                            .fill(BG2)
                            .stroke(Stroke::new(1.0, GRAY))
                            .inner_margin(8.0)
                            .show(ui, |ui| {
                                ui.set_min_width(120.0);
                                ui.label(RichText::new(label)
                                    .font(FontId::monospace(9.0))
                                    .color(CYAN_DIM));
                                ui.label(RichText::new(&value)
                                    .font(FontId::monospace(16.0))
                                    .color(color));
                            });
                        ui.add_space(6.0);
                    }
                });

                ui.add_space(10.0);

                // График
                ui.label(RichText::new("# ping_monitor --live")
                    .font(FontId::monospace(10.0))
                    .color(CYAN_DIM));
                ui.add_space(4.0);

                let plot_size = Vec2::new(ui.available_width(), 58.0);
                let (rect, _) = ui.allocate_exact_size(plot_size, egui::Sense::hover());
                let painter = ui.painter();
                painter.rect_filled(rect, 2.0, BG2);
                painter.rect_stroke(rect, 2.0, Stroke::new(1.0, GRAY));

                if self.ping_history.len() > 1 {
                    let n = self.ping_history.len();
                    let pts: Vec<egui::Pos2> = self.ping_history.iter().enumerate().map(|(i, &v)| {
                        let x = rect.left() + (i as f32 / (n-1) as f32) * rect.width();
                        let y = rect.bottom() - ((v - 10.0) / 40.0).clamp(0.0, 1.0) * rect.height();
                        egui::pos2(x, y)
                    }).collect();
                    for w in pts.windows(2) {
                        painter.line_segment([w[0], w[1]], Stroke::new(1.5, CYAN));
                    }
                }

                ui.add_space(10.0);

                // Серверы
                ui.label(RichText::new("# node_list --sort=latency")
                    .font(FontId::monospace(10.0))
                    .color(CYAN_DIM));
                ui.add_space(4.0);

                for (i, server) in SERVERS.iter().enumerate() {
                    let is_sel = self.selected_server == i;
                    let border_color = if is_sel { CYAN } else { GRAY };
                    let ping_color = if server.ping < 35.0 { CYAN } else { YELLOW };

                    let resp = egui::Frame::none()
                        .fill(BG2)
                        .stroke(Stroke::new(if is_sel { 1.5 } else { 1.0 }, border_color))
                        .inner_margin(8.0)
                        .show(ui, |ui| {
                            ui.horizontal(|ui| {
                                ui.label(RichText::new(if is_sel { "[*]" } else { "[ ]" })
                                    .font(FontId::monospace(11.0))
                                    .color(border_color));
                                ui.label(RichText::new(server.name)
                                    .font(FontId::monospace(11.0))
                                    .color(TEXT));
                                ui.label(RichText::new(server.sub)
                                    .font(FontId::monospace(9.0))
                                    .color(GRAY));
                                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                    ui.label(RichText::new(format!("{:.0}ms", server.ping))
                                        .font(FontId::monospace(11.0))
                                        .color(ping_color));
                                });
                            });
                        });

                    if resp.response.interact(egui::Sense::click()).clicked() {
                        self.selected_server = i;
                    }
                    ui.add_space(4.0);
                }
            });

        ctx.request_repaint();
    }
}