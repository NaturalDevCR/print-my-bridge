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
    match handle_auto_start_change(enable) {
        Ok(_) => {
            let mut config = crate::config::load_config().map_err(|e| e.to_string())?;
            config.auto_start = enable;
            save_config(&config).map_err(|e| e.to_string())?;
            Ok(())
        }
        Err(e) => {
            log::error!("Error cambiando auto-start: {}", e);
            Err(format!("Error al cambiar configuración de auto-inicio: {}", e))
        }
    }
}

fn handle_auto_start_change(enable: bool) -> Result<(), Box<dyn std::error::Error>> {
    #[cfg(target_os = "windows")]
    {
        use std::process::Command;
        
        let exe_path = std::env::current_exe()?;
        let app_name = "Print My Bridge";
        
        if enable {
            // Agregar a auto-inicio usando registro de Windows
            let output = Command::new("reg")
                .args([
                    "add",
                    "HKEY_CURRENT_USER\\Software\\Microsoft\\Windows\\CurrentVersion\\Run",
                    "/v",
                    app_name,
                    "/t",
                    "REG_SZ",
                    "/d",
                    &format!("\"{}\"", exe_path.display()),
                    "/f"
                ])
                .output()?;
                
            if !output.status.success() {
                return Err(format!("Failed to enable auto-start: {}", 
                    String::from_utf8_lossy(&output.stderr)).into());
            }
        } else {
            // Remover de auto-inicio
            let output = Command::new("reg")
                .args([
                    "delete",
                    "HKEY_CURRENT_USER\\Software\\Microsoft\\Windows\\CurrentVersion\\Run",
                    "/v",
                    app_name,
                    "/f"
                ])
                .output()?;
                
            // No es error si la clave no existe
            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                if !stderr.contains("The system was unable to find the specified registry key") {
                    return Err(format!("Failed to disable auto-start: {}", stderr).into());
                }
            }
        }
        
        Ok(())
    }
    
    #[cfg(not(target_os = "windows"))]
    {
        // Usar auto-launch para otros sistemas operativos
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