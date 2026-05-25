mod tunnel;
mod routing;

use std::sync::Arc;
use log::info;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::init();
    
    info!("RouteX starting...");
    
    // 1. Создаём wintun адаптер
    let adapter = tunnel::create_adapter()?;
    info!("TUN adapter created");
    
    // 2. Подключаемся к серверу
    let server_ip = "YOUR_VPS_IP:51820"; // временно хардкод
    tunnel::connect(adapter, server_ip).await?;
    
    Ok(())
}
