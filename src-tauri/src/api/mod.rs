use warp::{Filter, Reply};
use serde::{Deserialize, Serialize};
use crate::printer::PrinterManager;
use crate::error::BridgeError;
use crate::config::Config;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Deserialize)]
pub struct PrintRequest {
    pub printer_name: Option<String>,
    pub content: String,
    pub content_type: String, // "pdf", "html", "text", "image"
    pub copies: Option<u32>,
    pub options: Option<PrintOptions>,
}

#[derive(Deserialize)]
pub struct PrintOptions {
    pub paper_size: Option<String>,
    pub orientation: Option<String>,
    pub color: Option<bool>,
    pub duplex: Option<bool>,
}

#[derive(Serialize)]
pub struct PrintResponse {
    pub success: bool,
    pub message: String,
    pub job_id: Option<String>,
}

#[derive(Serialize)]
pub struct PrinterInfo {
    pub name: String,
    pub status: String,
    pub is_default: bool,
    pub supports_color: bool,
    pub paper_sizes: Vec<String>,
}

#[derive(Clone)]
pub struct SecurityContext {
    pub config: Arc<Config>,
    pub rate_limiter: Arc<Mutex<HashMap<String, Vec<u64>>>>,
}

pub fn routes(config: Config) -> impl Filter<Extract = impl Reply, Error = warp::Rejection> + Clone {
    let security_context = SecurityContext {
        config: Arc::new(config),
        rate_limiter: Arc::new(Mutex::new(HashMap::new())),
    };
    
    // Configurar CORS correctamente
    let cors = if security_context.config.allowed_origins.contains(&"*".to_string()) {
        // Si contiene "*", permitir cualquier origen
        warp::cors()
            .allow_any_origin()
            .allow_headers(vec!["content-type", "authorization", "x-api-token"])
            .allow_methods(vec!["GET", "POST", "OPTIONS"])
    } else {
        // Si no, usar los orígenes específicos (deben tener esquema completo)
        warp::cors()
            .allow_origins(security_context.config.allowed_origins.iter().map(|s| s.as_str()).collect::<Vec<_>>())
            .allow_headers(vec!["content-type", "authorization", "x-api-token"])
            .allow_methods(vec!["GET", "POST", "OPTIONS"])
    };
    
    let health = warp::path("health")
        .and(warp::get())
        .map(|| warp::reply::json(&serde_json::json!({
            "status": "ok",
            "service": "print-my-bridge",
            "version": env!("CARGO_PKG_VERSION")
        })));
    
    let auth_filter = warp::header::optional::<String>("x-api-token")
        .and(with_security_context(security_context.clone()))
        .and_then(validate_auth);
    
    let printers = warp::path!("api" / "printers")
        .and(warp::get())
        .and(auth_filter.clone())
        .and_then(get_printers);
    
    let print = warp::path!("api" / "print")
        .and(warp::post())
        .and(warp::body::content_length_limit(1024 * 1024 * 50)) // 50MB limit
        .and(warp::body::json())
        .and(auth_filter)
        .and_then(handle_print);
    
    health.or(printers).or(print).with(cors)
}

fn with_security_context(ctx: SecurityContext) -> impl Filter<Extract = (SecurityContext,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || ctx.clone())
}

async fn validate_auth(token: Option<String>, ctx: SecurityContext) -> Result<SecurityContext, warp::Rejection> {
    // Rate limiting
    let client_ip = "127.0.0.1".to_string(); // TODO: Get real IP
    let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
    
    {
        let mut limiter = ctx.rate_limiter.lock().unwrap();
        let requests = limiter.entry(client_ip).or_insert_with(Vec::new);
        
        // Remove old requests (older than 1 minute)
        requests.retain(|&time| now - time < 60);
        
        if requests.len() >= ctx.config.rate_limit_per_minute as usize {
            log::warn!("🚫 Rate limit exceeded for IP");
            return Err(warp::reject::custom(BridgeError::RateLimitExceeded));
        }
        
        requests.push(now);
    }
    
    // Token validation
    if let Some(required_token) = &ctx.config.api_token {
        match token {
            Some(provided_token) if provided_token == *required_token => {
                log::debug!("✅ Token válido");
                Ok(ctx)
            }
            _ => {
                log::warn!("🚫 Token inválido o faltante");
                Err(warp::reject::custom(BridgeError::Unauthorized))
            }
        }
    } else {
        Ok(ctx)
    }
}

async fn get_printers(_ctx: SecurityContext) -> Result<impl Reply, warp::Rejection> {
    match PrinterManager::get_available_printers().await {
        Ok(printers) => Ok(warp::reply::json(&printers)),
        Err(e) => {
            log::error!("Error obteniendo impresoras: {}", e);
            Err(warp::reject::custom(BridgeError::PrinterError(e.to_string())))
        }
    }
}

async fn handle_print(request: PrintRequest, ctx: SecurityContext) -> Result<impl Reply, warp::Rejection> {
    // Validar tipo de archivo
    if !ctx.config.allowed_file_types.contains(&request.content_type) {
        return Err(warp::reject::custom(BridgeError::UnsupportedFormat(request.content_type)));
    }
    
    // Validar tamaño (aproximado por base64)
    let estimated_size = (request.content.len() * 3) / 4; // base64 to bytes
    let max_size = (ctx.config.max_file_size_mb as usize) * 1024 * 1024;
    
    if estimated_size > max_size {
        log::warn!("🚫 Archivo demasiado grande: {} bytes", estimated_size);
        return Err(warp::reject::custom(BridgeError::FileTooLarge));
    }
    
    log::info!("📄 Nueva solicitud de impresión: {} ({} bytes)", request.content_type, estimated_size);
    
    match PrinterManager::print(request, &ctx.config).await {
        Ok(response) => Ok(warp::reply::json(&response)),
        Err(e) => {
            log::error!("Error en impresión: {}", e);
            Err(warp::reject::custom(BridgeError::PrintError(e.to_string())))
        }
    }
}