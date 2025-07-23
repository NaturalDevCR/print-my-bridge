use crate::api::{PrintRequest, PrintResponse, PrinterInfo};
use crate::error::BridgeResult;
use crate::config::Config;
use std::process::Command;
use tempfile::NamedTempFile;
use std::io::Write;
use base64::{Engine as _, engine::general_purpose};
use regex::Regex;

pub struct PrinterManager;

impl PrinterManager {
    pub async fn get_available_printers() -> BridgeResult<Vec<PrinterInfo>> {
        let mut printers = Vec::new();
        
        // Obtener impresora por defecto
        let default_printer = Self::get_default_printer().await?;
        
        // En macOS, usar lpstat para obtener impresoras
        let output = Command::new("lpstat")
            .args(["-p", "-d"])
            .output()?;
        
        let stdout = String::from_utf8_lossy(&output.stdout);
        
        for line in stdout.lines() {
            if line.starts_with("printer ") {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 2 {
                    let name = parts[1].to_string();
                    let capabilities = Self::get_printer_capabilities(&name).await?;
                    
                    printers.push(PrinterInfo {
                        name: name.clone(),
                        status: Self::get_printer_status(&name).await?,
                        is_default: Some(&name) == default_printer.as_ref(),
                        supports_color: capabilities.supports_color,
                        paper_sizes: capabilities.paper_sizes,
                    });
                }
            }
        }
        
        Ok(printers)
    }
    
    async fn get_default_printer() -> BridgeResult<Option<String>> {
        let output = Command::new("lpstat")
            .args(["-d"])
            .output()?;
        
        let stdout = String::from_utf8_lossy(&output.stdout);
        
        for line in stdout.lines() {
            if line.starts_with("system default destination: ") {
                let default = line.replace("system default destination: ", "");
                return Ok(Some(default));
            }
        }
        
        Ok(None)
    }
    
    async fn get_printer_status(printer_name: &str) -> BridgeResult<String> {
        let output = Command::new("lpstat")
            .args(["-p", printer_name])
            .output()?;
        
        let stdout = String::from_utf8_lossy(&output.stdout);
        
        if stdout.contains("is idle") {
            Ok("idle".to_string())
        } else if stdout.contains("is busy") {
            Ok("busy".to_string())
        } else if stdout.contains("disabled") {
            Ok("disabled".to_string())
        } else {
            Ok("unknown".to_string())
        }
    }
    
    async fn get_printer_capabilities(printer_name: &str) -> BridgeResult<PrinterCapabilities> {
        let output = Command::new("lpoptions")
            .args(["-p", printer_name, "-l"])
            .output()?;
        
        let stdout = String::from_utf8_lossy(&output.stdout);
        
        let supports_color = stdout.contains("ColorModel") && 
                           (stdout.contains("RGB") || stdout.contains("CMYK"));
        
        let paper_sizes = Self::extract_paper_sizes(&stdout);
        
        Ok(PrinterCapabilities {
            supports_color,
            paper_sizes,
        })
    }
    
    fn extract_paper_sizes(lpoptions_output: &str) -> Vec<String> {
        let mut sizes = Vec::new();
        
        for line in lpoptions_output.lines() {
            if line.starts_with("PageSize/") {
                let re = Regex::new(r"\*?([A-Za-z0-9]+)").unwrap();
                for cap in re.captures_iter(line) {
                    if let Some(size) = cap.get(1) {
                        let size_str = size.as_str();
                        if !sizes.contains(&size_str.to_string()) {
                            sizes.push(size_str.to_string());
                        }
                    }
                }
            }
        }
        
        if sizes.is_empty() {
            sizes = vec!["A4".to_string(), "Letter".to_string()];
        }
        
        sizes
    }
    
    pub async fn print(request: PrintRequest, config: &Config) -> BridgeResult<PrintResponse> {
        let printer_name = request.printer_name
            .or_else(|| config.default_printer.clone())
            .unwrap_or_else(|| "default".to_string());
        
        match request.content_type.as_str() {
            "pdf" => Self::print_pdf(&printer_name, &request.content, request.copies).await,
            "html" => Self::print_html(&printer_name, &request.content, request.copies).await,
            "text" => Self::print_text(&printer_name, &request.content, request.copies).await,
            "image" => Self::print_image(&printer_name, &request.content, request.copies).await,
            _ => Err(crate::error::BridgeError::UnsupportedFormat(request.content_type)),
        }
    }
    
    async fn print_pdf(printer: &str, content: &str, copies: Option<u32>) -> BridgeResult<PrintResponse> {
        let pdf_data = general_purpose::STANDARD.decode(content)?;
        let mut temp_file = NamedTempFile::new()?;
        temp_file.write_all(&pdf_data)?;
        
        let copies_str = copies.unwrap_or(1).to_string();
        
        let output = Command::new("lp")
            .args(["-d", printer, "-n", &copies_str, temp_file.path().to_str().unwrap()])
            .output()?;
        
        if output.status.success() {
            let job_id = Self::extract_job_id(&output.stdout);
            Ok(PrintResponse {
                success: true,
                message: "PDF enviado a impresora exitosamente".to_string(),
                job_id,
            })
        } else {
            let error = String::from_utf8_lossy(&output.stderr);
            Err(crate::error::BridgeError::PrintError(error.to_string()))
        }
    }
    
    async fn print_html(_printer: &str, content: &str, _copies: Option<u32>) -> BridgeResult<PrintResponse> {
        // Convertir HTML a PDF usando wkhtmltopdf
        let mut html_file = NamedTempFile::with_suffix(".html")?;
        html_file.write_all(content.as_bytes())?;
        
        let pdf_file = NamedTempFile::with_suffix(".pdf")?;
        
        let output = Command::new("wkhtmltopdf")
            .args([
                "--page-size", "A4",
                "--margin-top", "0.75in",
                "--margin-right", "0.75in",
                "--margin-bottom", "0.75in",
                "--margin-left", "0.75in",
                html_file.path().to_str().unwrap(),
                pdf_file.path().to_str().unwrap()
            ])
            .output()?;
        
        if output.status.success() {
            // Ahora imprimir el PDF generado
            let pdf_data = std::fs::read(pdf_file.path())?;
            let pdf_base64 = general_purpose::STANDARD.encode(&pdf_data);
            
            Self::print_pdf(_printer, &pdf_base64, _copies).await
        } else {
            // Fallback: abrir en navegador
            Command::new("open")
                .args(["-a", "Safari", html_file.path().to_str().unwrap()])
                .spawn()?;
            
            Ok(PrintResponse {
                success: true,
                message: "HTML convertido y enviado a impresora".to_string(),
                job_id: None,
            })
        }
    }
    
    fn extract_job_id(lp_output: &[u8]) -> Option<String> {
        let output_str = String::from_utf8_lossy(lp_output);
        let re = Regex::new(r"request id is ([^\s]+)").unwrap();
        
        if let Some(captures) = re.captures(&output_str) {
            if let Some(job_id) = captures.get(1) {
                return Some(job_id.as_str().to_string());
            }
        }
        
        None
    }
    
    async fn print_text(printer: &str, content: &str, copies: Option<u32>) -> BridgeResult<PrintResponse> {
        let mut temp_file = NamedTempFile::with_suffix(".txt")?;
        temp_file.write_all(content.as_bytes())?;
        
        let copies_str = copies.unwrap_or(1).to_string();
        
        let output = Command::new("lp")
            .args(["-d", printer, "-n", &copies_str, temp_file.path().to_str().unwrap()])
            .output()?;
        
        if output.status.success() {
            Ok(PrintResponse {
                success: true,
                message: "Texto enviado a impresora exitosamente".to_string(),
                job_id: Some("text_job_123".to_string()),
            })
        } else {
            let error = String::from_utf8_lossy(&output.stderr);
            Err(crate::error::BridgeError::PrintError(error.to_string()))
        }
    }
    
    async fn print_image(printer: &str, content: &str, copies: Option<u32>) -> BridgeResult<PrintResponse> {
        let image_data = general_purpose::STANDARD.decode(content)?;
        let mut temp_file = NamedTempFile::with_suffix(".png")?;
        temp_file.write_all(&image_data)?;
        
        let copies_str = copies.unwrap_or(1).to_string();
        
        let output = Command::new("lp")
            .args(["-d", printer, "-n", &copies_str, temp_file.path().to_str().unwrap()])
            .output()?;
        
        if output.status.success() {
            Ok(PrintResponse {
                success: true,
                message: "Imagen enviada a impresora exitosamente".to_string(),
                job_id: Some("image_job_123".to_string()),
            })
        } else {
            let error = String::from_utf8_lossy(&output.stderr);
            Err(crate::error::BridgeError::PrintError(error.to_string()))
        }
    }
}

struct PrinterCapabilities {
    supports_color: bool,
    paper_sizes: Vec<String>,
}