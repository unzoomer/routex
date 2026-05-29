use std::fs;
use std::path::PathBuf;

pub struct Config {
    pub private_key: String,
    pub server_addr: String,
    pub server_public_key: String,
}

impl Config {
    pub fn load_or_create() -> Self {
        let config_dir = Self::config_dir();
        fs::create_dir_all(&config_dir).ok();

        let key_path = config_dir.join("client.key");
        let private_key = if key_path.exists() {
            fs::read_to_string(&key_path)
                .unwrap_or_default()
                .trim()
                .to_string()
        } else {
            let key = Self::generate_key();
            fs::write(&key_path, &key).ok();
            log::info!("Generated new client key");
            let pubkey = Self::derive_public_key(&key);
            log::info!("Public key: {}", pubkey);
            Self::register_key_sync(&pubkey);
            key
        };

        Self {
            private_key,
            server_addr: "139.100.219.5:51820".to_string(),
            server_public_key: "s8qNGa7xgugqUQSpLEgiLRo6yrNRcAZFc3zPn5zQMmw=".to_string(),
        }
    }

    pub fn public_key(&self) -> String {
        Self::derive_public_key(&self.private_key)
    }

    fn derive_public_key(private_key: &str) -> String {
        use boringtun::x25519;
        use base64::{Engine as _, engine::general_purpose};
        let bytes = general_purpose::STANDARD
            .decode(private_key)
            .unwrap_or_default();
        if bytes.len() == 32 {
            let arr: [u8; 32] = bytes.try_into().unwrap();
            let secret = x25519::StaticSecret::from(arr);
            let public = x25519::PublicKey::from(&secret);
            general_purpose::STANDARD.encode(public.as_bytes())
        } else {
            String::new()
        }
    }

    fn generate_key() -> String {
        use boringtun::x25519;
        use base64::{Engine as _, engine::general_purpose};
        let mut bytes = [0u8; 32];
        for b in bytes.iter_mut() {
            *b = rand::random::<u8>();
        }
        let secret = x25519::StaticSecret::from(bytes);
        general_purpose::STANDARD.encode(secret.as_bytes())
    }

    fn register_key_sync(public_key: &str) {
        let url = "http://139.100.219.5:8080/api/register";
        let body = format!(r#"{{"public_key":"{}"}}"#, public_key);
        log::info!("Registering key with server...");
        let result = std::process::Command::new("curl")
            .args([
                "-s", "-X", "POST", url,
                "-H", "Content-Type: application/json",
                "-d", &body,
            ])
            .output();
        match result {
            Ok(out) => log::info!(
                "Register response: {}",
                String::from_utf8_lossy(&out.stdout)
            ),
            Err(e) => log::error!("Register failed: {}", e),
        }
    }

    fn config_dir() -> PathBuf {
        let mut path = dirs::data_local_dir()
            .unwrap_or_else(|| PathBuf::from("."));
        path.push("RouteX");
        path
    }
}