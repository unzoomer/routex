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
    selected_game: usize,
    game_detector: crate::games::GameDetector,
    running_games: Vec<&'static crate::games::Game>,
    private_key: String,
    server_addr: String,
    server_public_key: String,
    direct_ping: f32,
    direct_rx: Option<std::sync::mpsc::Receiver<f32>>,
    // Auth
    auth_email: String,
    auth_password: String,
    auth_token: Option<String>,
    auth_error: String,
    logged_in: bool,
    auth_screen: bool,
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
            selected_game: 0,
            game_detector: crate::games::GameDetector::new(),
            running_games: Vec::new(),
            private_key: String::new(),
            server_addr: "139.100.219.5:51820".to_string(),
            server_public_key: "s8qNGa7xgugqUQSpLEgiLRo6yrNRcAZFc3zPn5zQMmw=".to_string(),
            direct_ping: 0.0,
            direct_rx: None,
            auth_email: String::new(),
            auth_password: String::new(),
            auth_token: None,
            auth_error: String::new(),
            logged_in: false,
            auth_screen: true,
        }
    }
}

impl RouteXApp {
    pub fn new(cfg: crate::config::Config) -> Self {
        Self {
            private_key: cfg.private_key,
            server_addr: cfg.server_addr,
            server_public_key: cfg.server_public_key,
            ..Default::default()
        }
    }

    fn do_login(email: &str, password: &str) -> Result<String, String> {
        let body = format!(r#"{{"email":"{}","password":"{}"}}"#, email, password);
        let output = std::process::Command::new("curl")
            .args([
                "-s", "-X", "POST",
                "http://139.100.219.5:8081/auth/login",
                "-H", "Content-Type: application/json",
                "-d", &body,
            ])
            .output()
            .map_err(|e| e.to_string())?;
        let text = String::from_utf8_lossy(&output.stdout);
        let json: serde_json::Value = serde_json::from_str(&text)
            .map_err(|_| "Server error".to_string())?;
        if let Some(token) = json["token"].as_str() {
            Ok(token.to_string())
        } else if let Some(err) = json["error"].as_str() {
            Err(err.to_string())
        } else {
            Err("Unknown error".to_string())
        }
    }

    fn do_register(email: &str, password: &str) -> Result<(), String> {
        let body = format!(r#"{{"email":"{}","password":"{}"}}"#, email, password);
        let output = std::process::Command::new("curl")
            .args([
                "-s", "-X", "POST",
                "http://139.100.219.5:8081/auth/register",
                "-H", "Content-Type: application/json",
                "-d", &body,
            ])
            .output()
            .map_err(|e| e.to_string())?;
        let text = String::from_utf8_lossy(&output.stdout);
        let json: serde_json::Value = serde_json::from_str(&text)
            .map_err(|_| "Server error".to_string())?;
        if json["message"].as_str().is_some() {
            Ok(())
        } else if let Some(err) = json["error"].as_str() {
            Err(err.to_string())
        } else {
            Err("Unknown error".to_string())
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
const GREEN:    Color32 = Color32::from_rgb(0,   200, 100);

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

        if let Some(rx) = &self.ping_rx {
            if let Ok(ping) = rx.try_recv() {
                self.current_ping = ping;
                self.ping_history.push(ping);
                if self.ping_history.len() > 50 {
                    self.ping_history.remove(0);
                }
            }
        }

        if let Some(rx) = &self.direct_rx {
            if let Ok(ping) = rx.try_recv() {
                self.direct_ping = ping;
            }
        }

        if self.frame % 120 == 0 {
            self.running_games = self.game_detector.detect_running();
        }

        ctx.style_mut(|s| {
            s.visuals.panel_fill = BG;
            s.visuals.window_fill = BG;
            s.visuals.override_text_color = Some(TEXT);
            s.visuals.widgets.inactive.bg_fill = BG2;
            s.visuals.widgets.hovered.bg_fill = Color32::from_rgb(20, 30, 35);
            s.visuals.widgets.active.bg_fill = Color32::from_rgb(0, 40, 50);
        });

        // Экран логина
        if self.auth_screen {
            egui::CentralPanel::default()
                .frame(egui::Frame::none()
                    .fill(BG)
                    .inner_margin(egui::Margin::symmetric(80.0, 40.0)))
                .show(ctx, |ui| {
                    ui.vertical_centered(|ui| {
                        ui.add_space(30.0);
                        ui.horizontal(|ui| {
                            ui.add_space(ui.available_width() / 2.0 - 40.0);
                            ui.label(RichText::new("ROUTE")
                                .font(FontId::monospace(28.0))
                                .color(CYAN)
                                .strong());
                            ui.label(RichText::new("X")
                                .font(FontId::monospace(28.0))
                                .color(RED)
                                .strong());
                        });
                        ui.add_space(4.0);
                        ui.label(RichText::new("v0.1.0-alpha")
                            .font(FontId::monospace(10.0))
                            .color(CYAN_DIM));
                        ui.add_space(24.0);

                        ui.label(RichText::new("# login --interactive")
                            .font(FontId::monospace(11.0))
                            .color(CYAN_DIM));
                        ui.add_space(12.0);

                        ui.label(RichText::new("EMAIL")
                            .font(FontId::monospace(10.0))
                            .color(CYAN_DIM));
                        ui.add(egui::TextEdit::singleline(&mut self.auth_email)
                            .font(FontId::monospace(13.0))
                            .desired_width(260.0)
                            .hint_text("user@example.com"));
                        ui.add_space(8.0);

                        ui.label(RichText::new("PASSWORD")
                            .font(FontId::monospace(10.0))
                            .color(CYAN_DIM));
                        ui.add(egui::TextEdit::singleline(&mut self.auth_password)
                            .font(FontId::monospace(13.0))
                            .desired_width(260.0)
                            .password(true)
                            .hint_text("••••••••"));
                        ui.add_space(12.0);

                        if !self.auth_error.is_empty() {
                            let err_color = if self.auth_error.contains("Registered") { GREEN } else { RED };
                            ui.label(RichText::new(&self.auth_error)
                                .font(FontId::monospace(11.0))
                                .color(err_color));
                            ui.add_space(8.0);
                        }

                        ui.horizontal(|ui| {
                            ui.add_space(ui.available_width() / 2.0 - 100.0);

                            let login_btn = ui.add(egui::Button::new(
                                RichText::new("[ LOGIN ]")
                                    .font(FontId::monospace(12.0))
                                    .color(CYAN))
                                .fill(BG2)
                                .stroke(Stroke::new(1.0, CYAN)));

                            ui.add_space(8.0);

                            let reg_btn = ui.add(egui::Button::new(
                                RichText::new("[ REGISTER ]")
                                    .font(FontId::monospace(12.0))
                                    .color(GRAY))
                                .fill(BG2)
                                .stroke(Stroke::new(1.0, GRAY)));

                            if login_btn.clicked() {
                                let email = self.auth_email.clone();
                                let password = self.auth_password.clone();
                                match Self::do_login(&email, &password) {
                                    Ok(token) => {
                                        self.auth_token = Some(token);
                                        self.auth_screen = false;
                                        self.logged_in = true;
                                        self.auth_error.clear();
                                    }
                                    Err(e) => self.auth_error = e,
                                }
                            }

                            if reg_btn.clicked() {
                                let email = self.auth_email.clone();
                                let password = self.auth_password.clone();
                                match Self::do_register(&email, &password) {
                                    Ok(_) => self.auth_error = "Registered! Now login.".to_string(),
                                    Err(e) => self.auth_error = e,
                                }
                            }
                        });
                    });
                });
            ctx.request_repaint();
            return;
        }

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
                    ui.label(RichText::new(format!("USER   {}", self.auth_email))
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
                                let private_key_clone = self.private_key.clone();
                                let server_addr_clone = self.server_addr.clone();
                                let server_pubkey_clone = self.server_public_key.clone();

                                let (tx, rx) = std::sync::mpsc::channel::<bool>();
                                self.tunnel_tx = Some(tx);

                                std::thread::spawn(move || {
                                    let rt = tokio::runtime::Runtime::new().unwrap();
                                    rt.block_on(async {
                                        match crate::tunnel::create_adapter() {
                                            Ok(adapter) => {
                                                let tunnel = crate::tunnel::WireGuardTunnel::new(
                                                    adapter,
                                                    &server_addr_clone,
                                                    &private_key_clone,
                                                    &server_pubkey_clone,
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

                                let (ping_tx, ping_rx) = std::sync::mpsc::channel::<f32>();
                                self.ping_rx = Some(ping_rx);
                                let server_addr_ping = self.server_addr.clone();

                                std::thread::spawn(move || {
                                    loop {
                                        let sock = std::net::UdpSocket::bind("0.0.0.0:0").unwrap();
                                        sock.set_read_timeout(Some(
                                            std::time::Duration::from_millis(2000)
                                        )).unwrap();
                                        let ping_addr = server_addr_ping.replace(":51820", ":7777");
                                        let start = std::time::Instant::now();
                                        let _ = sock.send_to(b"ping", &ping_addr);
                                        let mut buf = [0u8; 4];
                                        if sock.recv(&mut buf).is_ok() {
                                            let ping = start.elapsed().as_millis() as f32;
                                            let _ = ping_tx.send(ping);
                                        }
                                        std::thread::sleep(std::time::Duration::from_secs(1));
                                    }
                                });

                                let (direct_tx, direct_rx) = std::sync::mpsc::channel::<f32>();
                                self.direct_rx = Some(direct_rx);

                                std::thread::spawn(move || {
                                    loop {
                                        if let Some(ping) = crate::latency::LatencyMeter::icmp_ping(
                                            "162.254.197.36"
                                        ) {
                                            let _ = direct_tx.send(ping);
                                        }
                                        std::thread::sleep(std::time::Duration::from_secs(5));
                                    }
                                });

                            } else {
                                self.tunnel_tx = None;
                                self.ping_rx = None;
                                self.direct_rx = None;
                                self.current_ping = 0.0;
                                self.direct_ping = 0.0;
                            }
                        }
                    });
                });

                ui.add_space(10.0);

                ui.horizontal(|ui| {
                    let improvement = if self.direct_ping > 0.0 && self.current_ping > 0.0 {
                        self.direct_ping - self.current_ping
                    } else {
                        0.0
                    };
                    let imp_color = if improvement > 0.0 { GREEN } else { GRAY };

                    let metrics: &[(&str, String, Color32)] = &[
                        ("ROUTEX", format!("{:.0}ms", if self.connected { self.current_ping } else { 0.0 }), CYAN),
                        ("DIRECT", format!("{:.0}ms", if self.connected { self.direct_ping } else { 0.0 }), GRAY),
                        ("SAVED",  format!("{:+.0}ms", if self.connected { improvement } else { 0.0 }), imp_color),
                    ];

                    for (label, value, color) in metrics {
                        egui::Frame::none()
                            .fill(BG2)
                            .stroke(Stroke::new(1.0, GRAY))
                            .inner_margin(8.0)
                            .show(ui, |ui| {
                                ui.set_min_width(120.0);
                                ui.label(RichText::new(*label)
                                    .font(FontId::monospace(9.0))
                                    .color(CYAN_DIM));
                                ui.label(RichText::new(value.as_str())
                                    .font(FontId::monospace(16.0))
                                    .color(*color));
                            });
                        ui.add_space(6.0);
                    }
                });

                ui.add_space(10.0);

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

                ui.label(RichText::new("# game_detect --auto")
                    .font(FontId::monospace(10.0))
                    .color(CYAN_DIM));
                ui.add_space(4.0);

                if self.running_games.is_empty() {
                    egui::Frame::none()
                        .fill(BG2)
                        .stroke(Stroke::new(1.0, GRAY))
                        .inner_margin(8.0)
                        .show(ui, |ui| {
                            ui.label(RichText::new("[ ] no games detected — launch a game")
                                .font(FontId::monospace(11.0))
                                .color(GRAY));
                        });
                } else {
                    for (i, game) in self.running_games.iter().enumerate() {
                        let is_sel = self.selected_game == i;
                        let border_color = if is_sel { CYAN } else { GRAY };
                        let resp = egui::Frame::none()
                            .fill(BG2)
                            .stroke(Stroke::new(if is_sel { 1.5 } else { 1.0 }, border_color))
                            .inner_margin(8.0)
                            .show(ui, |ui| {
                                ui.horizontal(|ui| {
                                    ui.label(RichText::new(if is_sel { "[*]" } else { "[ ]" })
                                        .font(FontId::monospace(11.0))
                                        .color(border_color));
                                    ui.label(RichText::new(game.name)
                                        .font(FontId::monospace(11.0))
                                        .color(CYAN));
                                    ui.label(RichText::new(game.process)
                                        .font(FontId::monospace(9.0))
                                        .color(GRAY));
                                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                        ui.label(RichText::new("● RUNNING")
                                            .font(FontId::monospace(9.0))
                                            .color(GREEN));
                                    });
                                });
                            });
                        if resp.response.interact(egui::Sense::click()).clicked() {
                            self.selected_game = i;
                        }
                        ui.add_space(4.0);
                    }
                }

                ui.add_space(8.0);

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