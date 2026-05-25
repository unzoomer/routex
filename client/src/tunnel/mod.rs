use wintun;
use std::sync::Arc;
use log::{info, error};

pub fn create_adapter() -> anyhow::Result<Arc<wintun::Adapter>> {
    // Загружаем wintun.dll
    let wintun = unsafe {
        wintun::load_from_path("wintun.dll")
            .expect("Failed to load wintun.dll")
    };

    // Создаём виртуальный сетевой адаптер
    let adapter = match wintun::Adapter::open(&wintun, "RouteX") {
        Ok(a) => {
            info!("Opened existing RouteX adapter");
            a
        }
        Err(_) => {
            info!("Creating new RouteX adapter...");
            wintun::Adapter::create(&wintun, "RouteX", "RouteX Tunnel", None)
                .expect("Failed to create adapter")
        }
    };

    Ok(Arc::new(adapter))
}

pub async fn connect(
    adapter: Arc<wintun::Adapter>,
    server_addr: &str,
) -> anyhow::Result<()> {
    use std::net::Ipv4Addr;

    // Назначаем IP виртуальному адаптеру
    let tun_ip: Ipv4Addr = "10.0.0.2".parse()?;
    info!("TUN IP: {}", tun_ip);

    // Создаём сессию (буфер 4MB)
    let session = Arc::new(
        adapter.start_session(wintun::MAX_RING_CAPACITY)?
    );

    info!("Connected to session, ready to forward packets");

    // Читаем пакеты из адаптера и пересылаем на сервер
    let session_reader = session.clone();
    let session_writer = session.clone();

    // Поток чтения (игра → туннель)
    let read_handle = tokio::spawn(async move {
        loop {
            match session_reader.receive_blocking() {
                Ok(packet) => {
                    // TODO: зашифровать и отправить на VPS
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

    // Поток записи (сервер → игра)
    let write_handle = tokio::spawn(async move {
        // TODO: получать пакеты с VPS и писать в адаптер
        loop {
            tokio::time::sleep(
                tokio::time::Duration::from_secs(1)
            ).await;
        }
    });

    read_handle.await?;
    write_handle.await?;

    Ok(())
}
