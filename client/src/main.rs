mod tunnel;
mod routing;
mod ui;

use log::info;

// Конфиг WireGuard — замени на свои ключи
const SERVER_ADDR: &str = "139.100.219.5:51820";
const CLIENT_PRIVATE_KEY: &str = "0NRGDaPPMSwY8qPMUP9Nx+gbh5nAyKEtKU58rzzv3Hk=";
const SERVER_PUBLIC_KEY: &str = "s8qNGa7xgugqUQSpLEgiLRo6yrNRcAZFc3zPn5zQMmw=";

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::init();
    info!("RouteX starting...");

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
        Box::new(|_cc| Box::new(ui::RouteXApp::default())),
    ).unwrap();

    Ok(())
}