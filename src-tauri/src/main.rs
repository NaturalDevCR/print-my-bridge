// Ocultar consola en Windows para release builds
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod api;
mod printer;
mod config;
mod error;
mod gui;

use warp::Filter;
use std::env;
use tauri::{Manager, WindowEvent, tray::{TrayIconBuilder, TrayIconEvent}, menu::{MenuBuilder, MenuItemBuilder}};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Inicializar logging solo en debug
    #[cfg(debug_assertions)]
    env_logger::init();
    
    // Cargar configuración de forma asíncrona
    let config = tokio::task::spawn_blocking(|| config::load_config()).await??;
    
    #[cfg(debug_assertions)]
    log::info!("🚀 Iniciando Print My Bridge v{}", env!("CARGO_PKG_VERSION"));
    
    // Verificar si se debe ejecutar en modo GUI o headless
    let args: Vec<String> = env::args().collect();
    let headless_mode = args.contains(&"--headless".to_string());
    
    if headless_mode {
        start_http_server(config).await?;
    } else {
        start_gui_app(config).await?;
    }
    
    Ok(())
}

async fn start_http_server(config: config::Config) -> Result<(), Box<dyn std::error::Error>> {
    // Configurar CORS
    let cors = warp::cors()
        .allow_any_origin()
        .allow_headers(vec!["content-type", "authorization", "x-api-token"])
        .allow_methods(vec!["GET", "POST", "PUT", "DELETE", "OPTIONS"]);
    
    // Rutas de la API
    let api_routes = api::routes(config.clone())
        .with(cors)
        .with(warp::log("print_my_bridge"));
    
    // Iniciar servidor
    warp::serve(api_routes)
        .run(([127, 0, 0, 1], config.port))
        .await;
    
    Ok(())
}

async fn start_gui_app(config: config::Config) -> Result<(), Box<dyn std::error::Error>> {
    // Iniciar servidor HTTP en background
    let config_clone = config.clone();
    let _server_handle = tokio::spawn(async move {
        log::info!("🚀 Iniciando servidor HTTP en background...");
        if let Err(e) = start_http_server(config_clone).await {
            log::error!("❌ Error crítico en servidor HTTP: {}", e);
            eprintln!("❌ Error crítico en servidor HTTP: {}", e);
        } else {
            log::info!("✅ Servidor HTTP iniciado correctamente");
        }
    });

    // Dar tiempo al servidor para iniciarse
    tokio::time::sleep(tokio::time::Duration::from_millis(1000)).await;
    
    // Verificar si el servidor se inició correctamente
    let config_test = config.clone();
    tokio::spawn(async move {
        tokio::time::sleep(tokio::time::Duration::from_millis(2000)).await;
        let client = reqwest::Client::new();
        match client.get(&format!("http://{}:{}/health", config_test.host, config_test.port)).send().await {
            Ok(response) if response.status().is_success() => {
                log::info!("✅ Servidor HTTP respondiendo correctamente en puerto {}", config_test.port);
            }
            Ok(response) => {
                log::error!("⚠️ Servidor HTTP respondió con estado: {}", response.status());
            }
            Err(e) => {
                log::error!("❌ No se puede conectar al servidor HTTP: {}", e);
            }
        }
    });

    // Iniciar aplicación Tauri
    tauri::Builder::default()
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_shell::init())
        .setup(|app| {
            // Crear menú del tray
            let show = MenuItemBuilder::with_id("show", "Mostrar").build(app)?;
            let hide = MenuItemBuilder::with_id("hide", "Ocultar").build(app)?;
            let quit = MenuItemBuilder::with_id("quit", "Cerrar").build(app)?;
            
            let menu = MenuBuilder::new(app)
                .items(&[&show, &hide])
                .separator()
                .item(&quit)
                .build()?;
            
            // Crear tray icon SOLO si no existe uno ya
            if app.tray_by_id("main-tray").is_none() {
                let _tray = TrayIconBuilder::new()
                    .menu(&menu)
                    .icon(app.default_window_icon().unwrap().clone())
                    .tooltip("Print My Bridge")
                    .on_menu_event(move |app, event| match event.id.as_ref() {
                        "quit" => {
                            app.exit(0);
                        }
                        "show" => {
                            if let Some(window) = app.get_webview_window("main") {
                                let _ = window.show();
                                let _ = window.set_focus();
                            }
                        }
                        "hide" => {
                            if let Some(window) = app.get_webview_window("main") {
                                let _ = window.hide();
                            }
                        }
                        _ => {}
                    })
                    .on_tray_icon_event(|tray, event| {
                        if let TrayIconEvent::Click { button: tauri::tray::MouseButton::Left, .. } = event {
                            let app = tray.app_handle();
                            if let Some(window) = app.get_webview_window("main") {
                                if window.is_visible().unwrap_or(false) {
                                    let _ = window.hide();
                                } else {
                                    let _ = window.show();
                                    let _ = window.set_focus();
                                }
                            }
                        }
                    })
                    .build(app)?;
            }
            
            Ok(())
        })
        .on_window_event(|window, event| match event {
            WindowEvent::CloseRequested { api, .. } => {
                // Prevenir cierre y minimizar al tray en su lugar
                let _ = window.hide();
                api.prevent_close();
            }
            _ => {}
        })
        .invoke_handler(tauri::generate_handler![
            gui::get_config,
            gui::update_config,
            gui::generate_new_token,
            gui::get_bridge_status,
            gui::toggle_auto_start
        ])
        .run(tauri::generate_context!())
        .expect("Error ejecutando aplicación Tauri");
    
    Ok(())
}