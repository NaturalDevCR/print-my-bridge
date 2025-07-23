use crate::config::{Config, save_config, generate_secure_token};
use serde::{Deserialize, Serialize};
use tauri::command;
use auto_launch::AutoLaunchBuilder;

#[derive(Serialize, Deserialize)]
pub struct BridgeStatus {
    pub active: bool,
    pub port: u16,
    pub version: String,
    pub requests_processed: u32,
}

#[command]
pub async fn get_config() -> Result<Config, String> {
    crate::config::load_config().map_err(|e| e.to_string())
}

#[command]
pub async fn update_config(config: Config) -> Result<(), String> {
    let old_config = crate::config::load_config().map_err(|e| e.to_string())?;
    
    // Manejar cambios en auto-inicio
    if config.auto_start != old_config.auto_start {
        handle_auto_start_change(config.auto_start).map_err(|e| e.to_string())?;
    }
    
    save_config(&config).map_err(|e| e.to_string())
}

#[command]
pub async fn toggle_auto_start(enable: bool) -> Result<(), String> {
    handle_auto_start_change(enable).map_err(|e| e.to_string())?;
    
    let mut config = crate::config::load_config().map_err(|e| e.to_string())?;
    config.auto_start = enable;
    save_config(&config).map_err(|e| e.to_string())
}

fn handle_auto_start_change(enable: bool) -> Result<(), Box<dyn std::error::Error>> {
    let auto = AutoLaunchBuilder::new()
        .set_app_name("Print My Bridge")
        .set_app_path(std::env::current_exe()?.to_str().ok_or("Invalid path")?)
        .build()?;
    
    if enable {
        auto.enable()?;
    } else {
        auto.disable()?;
    }
    
    Ok(())
}

#[command]
pub async fn generate_new_token() -> Result<String, String> {
    let mut config = crate::config::load_config().map_err(|e| e.to_string())?;
    let new_token = generate_secure_token();
    config.api_token = Some(new_token.clone());
    save_config(&config).map_err(|e| e.to_string())?;
    Ok(new_token)
}

#[command]
pub async fn get_bridge_status() -> Result<BridgeStatus, String> {
    let config = crate::config::load_config().map_err(|e| e.to_string())?;
    
    // Verificar si el servidor está activo
    let client = reqwest::Client::new();
    let is_active = match client
        .get(&format!("http://{}:{}/health", config.host, config.port))
        .send()
        .await
    {
        Ok(response) => response.status().is_success(),
        Err(_) => false,
    };
    
    Ok(BridgeStatus {
        active: is_active,
        port: config.port,
        version: env!("CARGO_PKG_VERSION").to_string(),
        requests_processed: 0, // TODO: Implementar contador real
    })
}