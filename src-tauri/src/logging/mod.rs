use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};
use tracing_appender::rolling::{RollingFileAppender, Rotation};
use std::path::PathBuf;
use std::fs;

pub struct LoggingConfig {
    pub level: String,
    pub file_enabled: bool,
    pub console_enabled: bool,
    pub json_format: bool,
    pub log_dir: PathBuf,
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            level: "info".to_string(),
            file_enabled: true,
            console_enabled: true,
            json_format: false,
            log_dir: get_default_log_dir(),
        }
    }
}

pub fn init_logging(config: Option<LoggingConfig>) -> Result<(), Box<dyn std::error::Error>> {
    let config = config.unwrap_or_default();
    
    // Crear directorio de logs si no existe
    if config.file_enabled {
        fs::create_dir_all(&config.log_dir)?;
    }
    
    // Configurar filtro de nivel
    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new(&config.level));
    
    let mut layers = Vec::new();
    
    // Capa de consola
    if config.console_enabled {
        if config.json_format {
            let console_layer = tracing_subscriber::fmt::layer()
                .json()
                .with_target(true)
                .with_thread_ids(true)
                .with_file(true)
                .with_line_number(true);
            layers.push(console_layer.boxed());
        } else {
            let console_layer = tracing_subscriber::fmt::layer()
                .pretty()
                .with_target(true)
                .with_thread_ids(true)
                .with_file(true)
                .with_line_number(true);
            layers.push(console_layer.boxed());
        }
    }
    
    // Capa de archivo
    if config.file_enabled {
        let file_appender = RollingFileAppender::new(
            Rotation::daily(),
            &config.log_dir,
            "print-my-bridge.log"
        );
        
        let file_layer = tracing_subscriber::fmt::layer()
            .json()
            .with_writer(file_appender)
            .with_target(true)
            .with_thread_ids(true)
            .with_file(true)
            .with_line_number(true);
        layers.push(file_layer.boxed());
    }
    
    // Inicializar subscriber
    tracing_subscriber::registry()
        .with(env_filter)
        .with(layers)
        .init();
    
    tracing::info!("🚀 Sistema de logging inicializado");
    tracing::info!("📁 Directorio de logs: {}", config.log_dir.display());
    tracing::info!("📊 Nivel de logging: {}", config.level);
    
    Ok(())
}

fn get_default_log_dir() -> PathBuf {
    #[cfg(target_os = "windows")]
    {
        dirs::data_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("print-my-bridge")
            .join("logs")
    }
    
    #[cfg(target_os = "macos")]
    {
        dirs::home_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("Library")
            .join("Logs")
            .join("print-my-bridge")
    }
    
    #[cfg(target_os = "linux")]
    {
        dirs::data_local_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("print-my-bridge")
            .join("logs")
    }
    
    #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
    {
        PathBuf::from("./logs")
    }
}

// Macros de conveniencia para logging estructurado
#[macro_export]
macro_rules! log_request {
    ($method:expr, $path:expr, $status:expr) => {
        tracing::info!(
            method = $method,
            path = $path,
            status = $status,
            "HTTP Request"
        );
    };
}

#[macro_export]
macro_rules! log_print_job {
    ($job_id:expr, $printer:expr, $status:expr) => {
        tracing::info!(
            job_id = $job_id,
            printer = $printer,
            status = $status,
            "Print Job"
        );
    };
}

#[macro_export]
macro_rules! log_error {
    ($error:expr, $context:expr) => {
        tracing::error!(
            error = %$error,
            context = $context,
            "Application Error"
        );
    };
}