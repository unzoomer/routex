#![allow(dead_code)]
use wintun;
use std::sync::Arc;
use std::net::UdpSocket;
use log::{info, error, debug};
use boringtun::noise::{Tunn, TunnResult};
use boringtun::x25519;
use base64::{Engine as _, engine::general_purpose};

pub struct WireGuardTunnel {
    adapter: Arc<wintun::Adapter>,
    server_addr: String,
    private_key: String,
    server_public_key: String,
}

impl WireGuardTunnel {
    pub fn new(
        adapter: Arc<wintun::Adapter>,
        server_addr: &str,
        private_key: &str,
        server_public_key: &str,
    ) -> Self {
        Self {
            adapter,
            server_addr: server_addr.to_string(),
            private_key: private_key.to_string(),
            server_public_key: server_public_key.to_string(),
        }
    }

    pub async fn run(&self) -> anyhow::Result<()> {
        let privkey_bytes = general_purpose::STANDARD
            .decode(&self.private_key)?;
        let pubkey_bytes = general_purpose::STANDARD
            .decode(&self.server_public_key)?;

        let private_key = x25519::StaticSecret::from(
            <[u8; 32]>::try_from(privkey_bytes.as_slice())?
        );
        let server_public = x25519::PublicKey::from(
            <[u8; 32]>::try_from(pubkey_bytes.as_slice())?
        );

        let tun = Tunn::new(
            private_key,
            server_public,
            None,
            Some(25),
            0,
            None,
        ).map_err(|e| anyhow::anyhow!("WG init error: {}", e))?;

        let tun = Arc::new(std::sync::Mutex::new(tun));
        let socket = UdpSocket::bind("0.0.0.0:0")?;
        socket.connect(&self.server_addr)?;
        let socket = Arc::new(socket);

        info!("WireGuard tunnel connecting to {}", self.server_addr);

        let session = Arc::new(
            self.adapter.start_session(wintun::MAX_RING_CAPACITY)?
        );

        info!("Session started, forwarding packets...");

        let tun_send = tun.clone();
        let socket_send = socket.clone();
        let session_read = session.clone();

        let send_handle = std::thread::spawn(move || {
            let mut buf = vec![0u8; 65535];
            loop {
                match session_read.receive_blocking() {
                    Ok(packet) => {
                        let data = packet.bytes().to_vec();
                        debug!("wintun → VPS: {} bytes", data.len());
                        let mut tun = tun_send.lock().unwrap();
                        match tun.encapsulate(&data, &mut buf) {
                            TunnResult::WriteToNetwork(encrypted) => {
                                if let Err(e) = socket_send.send(encrypted) {
                                    error!("Send error: {}", e);
                                }
                            }
                            _ => {}
                        }
                    }
                    Err(e) => {
                        error!("wintun read error: {}", e);
                        break;
                    }
                }
            }
        });

        let tun_recv = tun.clone();
        let socket_recv = socket.clone();
        let session_write = session.clone();

        let recv_handle = std::thread::spawn(move || {
            let mut buf = vec![0u8; 65535];
            let mut out = vec![0u8; 65535];
            loop {
                match socket_recv.recv(&mut buf) {
                    Ok(n) => {
                        debug!("VPS → wintun: {} bytes", n);
                        let mut tun = tun_recv.lock().unwrap();
                        match tun.decapsulate(None, &buf[..n], &mut out) {
                            TunnResult::WriteToTunnelV4(data, _) |
                            TunnResult::WriteToTunnelV6(data, _) => {
                                if let Ok(mut pkt) = session_write.allocate_send_packet(data.len() as u16) {
                                    pkt.bytes_mut().copy_from_slice(data);
                                    session_write.send_packet(pkt);
                                }
                            }
                            _ => {}
                        }
                    }
                    Err(e) => {
                        error!("Recv error: {}", e);
                        break;
                    }
                }
            }
        });

        send_handle.join().ok();
        recv_handle.join().ok();

        Ok(())
    }
}

pub fn create_adapter() -> anyhow::Result<Arc<wintun::Adapter>> {
    let wintun_lib = unsafe {
        wintun::load_from_path("wintun.dll")
            .expect("Failed to load wintun.dll")
    };

    let adapter = wintun::Adapter::create(
        &wintun_lib, "RouteX", "RouteX Tunnel", None
    ).expect("Failed to create adapter");

    Ok(Arc::new(adapter))
}