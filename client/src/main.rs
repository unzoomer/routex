mod tunnel;
mod routing;
mod ui;
mod games;
mod config;

use tray_icon::{TrayIconBuilder, menu::{Menu, MenuItem}};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::init();

    // Загружаем или создаём конфиг
    let cfg = config::Config::load_or_create();
    log::info!("Client public key: {}", cfg.public_key());

    let tray_menu = Menu::new();
    let show_item = MenuItem::new("Открыть RouteX", true, None);
    let quit_item = MenuItem::new("Выход", true, None);
    tray_menu.append(&show_item).unwrap();
    tray_menu.append(&quit_item).unwrap();

    let icon = create_tray_icon();
    let _tray_icon = TrayIconBuilder::new()
        .with_menu(Box::new(tray_menu))
        .with_tooltip("RouteX")
        .with_icon(icon)
        .build()
        .unwrap();

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("RouteX")
            .with_inner_size([720.0, 480.0])
            .with_min_inner_size([720.0, 480.0]),
        ..Default::default()
    };

    eframe::run_native(
        "RouteX",
        options,
        Box::new(move |_cc| Box::new(ui::RouteXApp::new(cfg))),
    ).unwrap();

    Ok(())
}

fn create_tray_icon() -> tray_icon::Icon {
    let width = 32u32;
    let height = 32u32;
    let mut rgba = vec![0u8; (width * height * 4) as usize];
    for y in 0..height {
        for x in 0..width {
            let idx = ((y * width + x) * 4) as usize;
            let cx = x as f32 - 16.0;
            let cy = y as f32 - 16.0;
            let dist = (cx * cx + cy * cy).sqrt();
            if dist < 14.0 && dist > 10.0 {
                rgba[idx] = 0; rgba[idx+1] = 240; rgba[idx+2] = 255; rgba[idx+3] = 255;
            } else if dist < 8.0 && (cx.abs() - cy.abs()).abs() < 2.5 {
                rgba[idx] = 255; rgba[idx+1] = 0; rgba[idx+2] = 60; rgba[idx+3] = 255;
            }
        }
    }
    tray_icon::Icon::from_rgba(rgba, width, height).unwrap()
}