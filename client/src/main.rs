mod tunnel;
mod routing;
mod ui;

use log::info;

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
        Box::new(|_cc| Ok(Box::new(ui::RouteXApp::default()))),
    ).unwrap();

    Ok(())
}