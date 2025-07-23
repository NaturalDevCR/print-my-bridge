use serde::{Deserialize, Serialize};
use crate::error::BridgeResult;
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Config {
    pub host: String,
    pub port: u16,
    pub max_file_size_mb: u64,
    pub rate_limit_per_minute: u32,
    pub api_token: Option<String>,
    pub auto_start: bool,
    pub minimize_to_tray: bool,
    // Campos faltantes añadidos:
    pub allowed_origins: Vec<String>,
    pub allowed_file_types: Vec<String>,
    pub default_printer: Option<String>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            host: "127.0.0.1".to_string(),
            port: 8765,
            max_file_size_mb: 50,
            rate_limit_per_minute: 60,
            api_token: None,
            auto_start: false,
            minimize_to_tray: true,
            // Valores por defecto para los nuevos campos:
            allowed_origins: vec!["*".to_string()],
            allowed_file_types: vec![
                "pdf".to_string(),
                "html".to_string(),
                "text".to_string(),
                "image".to_string()
            ],
            default_printer: None,
        }
    }
}

pub fn load_config() -> BridgeResult<Config> {
    let config_path = "print-my-bridge.toml";
    
    if Path::new(config_path).exists() {
        let config_str = fs::read_to_string(config_path)?;
        let config: Config = toml::from_str(&config_str)
            .map_err(|e| crate::error::BridgeError::ConfigError(e.to_string()))?;
        log::info!("📄 Configuración cargada desde {}", config_path);
        Ok(config)
    } else {
        let config = Config::default();
        save_config(&config)?;
        log::info!("📄 Configuración por defecto creada en {}", config_path);
        Ok(config)
    }
}

pub fn save_config(config: &Config) -> BridgeResult<()> {
    let config_str = toml::to_string_pretty(config)
        .map_err(|e| crate::error::BridgeError::ConfigError(e.to_string()))?;
    fs::write("print-my-bridge.toml", config_str)?;
    Ok(())
}

pub fn generate_secure_token() -> String {
    use rand::Rng;
    const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ\
                            abcdefghijklmnopqrstuvwxyz\
                            0123456789";
    let mut rng = rand::thread_rng();
    
    (0..32)
        .map(|_| {
            let idx = rng.gen_range(0..CHARSET.len());
            CHARSET[idx] as char
        })
        .collect()
}