use std::net::UdpSocket;
use std::time::{Duration, Instant};

#[derive(Debug, Clone)]
pub struct GameServer {
    pub game: &'static str,
    pub region: &'static str,
    pub addr: &'static str,
    pub port: u16,
}

pub const CS2_SERVERS: &[GameServer] = &[
    GameServer { game: "CS2", region: "EU-West",    addr: "185.25.182.1",   port: 27015 },
    GameServer { game: "CS2", region: "EU-Central", addr: "185.25.182.100", port: 27015 },
    GameServer { game: "CS2", region: "EU-East",    addr: "185.25.182.200", port: 27015 },
];

pub struct LatencyMeter;

impl LatencyMeter {
    pub fn udp_ping(addr: &str, port: u16, timeout_ms: u64) -> Option<f32> {
        let sock = UdpSocket::bind("0.0.0.0:0").ok()?;
        sock.set_read_timeout(Some(Duration::from_millis(timeout_ms))).ok()?;
        let target = format!("{}:{}", addr, port);
        let payload = b"\xff\xff\xff\xffTSource Engine Query\x00";
        let start = Instant::now();
        sock.send_to(payload, &target).ok()?;
        let mut buf = [0u8; 1400];
        match sock.recv(&mut buf) {
            Ok(_) => Some(start.elapsed().as_millis() as f32),
            Err(_) => Some(start.elapsed().as_millis() as f32),
        }
    }

    pub fn icmp_ping(addr: &str) -> Option<f32> {
        let output = std::process::Command::new("ping")
            .args(["-n", "1", "-w", "2000", addr])
            .output()
            .ok()?;
        let text = String::from_utf8_lossy(&output.stdout);
        for line in text.lines() {
            let lower = line.to_lowercase();
            if lower.contains("average") || lower.contains("среднее") || lower.contains("сред") {
                if let Some(ms) = Self::parse_ms(line) {
                    return Some(ms);
                }
            }
        }
        None
    }

    fn parse_ms(line: &str) -> Option<f32> {
        let line = line.to_lowercase();
        let idx = line.find("ms").or_else(|| line.find("мс"))?;
        let before = &line[..idx];
        let num_str: String = before.chars().rev()
            .take_while(|c| c.is_ascii_digit())
            .collect::<String>()
            .chars().rev().collect();
        num_str.parse().ok()
    }

    pub fn best_ping(addr: &str, port: u16) -> Option<f32> {
        let addr1 = addr.to_string();
        let addr2 = addr.to_string();
        let udp = std::thread::spawn(move || Self::udp_ping(&addr1, port, 2000));
        let icmp = std::thread::spawn(move || Self::icmp_ping(&addr2));
        let u = udp.join().ok().flatten();
        let i = icmp.join().ok().flatten();
        match (u, i) {
            (Some(a), Some(b)) => Some(a.min(b)),
            (Some(a), None) => Some(a),
            (None, Some(b)) => Some(b),
            (None, None) => None,
        }
    }
}