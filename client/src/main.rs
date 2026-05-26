mod tunnel;
mod routing;

use log::info;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::init();

    info!("RouteX starting...");

    let adapter = tunnel::create_adapter()?;
    info!("TUN adapter created");

    let server_ip = "127.0.0.1:51820";
    tunnel::connect(adapter, server_ip).await?;

    Ok(())
}