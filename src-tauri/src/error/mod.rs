use thiserror::Error;
use warp::reject::Reject;

pub type BridgeResult<T> = Result<T, BridgeError>;

#[derive(Error, Debug)]
pub enum BridgeError {
    #[error("Error de impresora: {0}")]
    PrinterError(String),
    
    #[error("Error de impresión: {0}")]
    PrintError(String),
    
    #[error("Formato no soportado: {0}")]
    UnsupportedFormat(String),
    
    #[error("Error de IO: {0}")]
    IoError(#[from] std::io::Error),
    
    #[error("Error de decodificación base64: {0}")]
    Base64Error(#[from] base64::DecodeError),
    
    #[error("Error de configuración: {0}")]
    ConfigError(String),
    
    #[error("No autorizado")]
    Unauthorized,
    
    #[error("Límite de velocidad excedido")]
    RateLimitExceeded,
    
    #[error("Archivo demasiado grande")]
    FileTooLarge,
}

impl Reject for BridgeError {}