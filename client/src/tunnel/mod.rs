use wintun;
use std::sync::Arc;
use log::info;

pub fn create_adapter() -> anyhow::Result<Arc<wintun::Adapter>> {
    let wintun_lib = unsafe {
        wintun::load_from_path("wintun.dll")
            .expect("Failed to load wintun.dll")
    };

    let adapter = match wintun::Adapter::open(&wintun_lib, "RouteX") {
        Ok(a) => {
            info!("Opened existing RouteX adapter");
            a
        }
        Err(_) => {
            info!("Creating new RouteX adapter...");
            wintun::Adapter::create(&wintun_lib, "RouteX", "RouteX Tunnel", None)
                .expect("Failed to create adapter")
        }
    };

    Ok(adapter)
}

pub async fn connect(
    adapter: Arc<wintun::Adapter>,
    _server_addr: &str,
) -> anyhow::Result<()> {
    let session = Arc::new(
        adapter.start_session(wintun::MAX_RING_CAPACITY)?
    );

    info!("Session started, capturing packets...");

    let session_reader = session.clone();

    let read_handle = tokio::spawn(async move {
        loop {
            match session_reader.receive_blocking() {
                Ok(packet) => {
                    log::debug!(
                        "Captured packet: {} bytes",
                        packet.bytes().len()
                    );
                }
                Err(e) => {
                    log::error!("Read error: {}", e);
                    break;
                }
            }
        }
    });

    read_handle.await?;
    Ok(())
}