use crate::compression::Compression;
use crate::error::{Result, ServerError};
use crate::request::{HttpMethod, HttpRequest};
use crate::response::HttpResponse;
use serde_json::json;
use std::fs;
use std::path::{Path, PathBuf};

/// Router handles incoming requests and generates responses
pub struct Router {
    pub file_directory: String,
}

impl Router {
    pub fn new(file_directory: String) -> Self {
        Router { file_directory }
    }

    /// Route an incoming request to the appropriate handler
    pub fn route(&self, request: HttpRequest) -> Result<Vec<u8>> {
        log::info!(
            "{} {} - {} bytes",
            request.method.as_str(),
            request.path,
            request.body.len()
        );

        // Determine compression
        let compression = if request.body.len() > 100 || request.path.starts_with("/echo/") {
            Compression::from_accept_encoding(&request.get_accepted_encodings())
        } else {
            Compression::None
        };

        let response = match (&request.method, request.path.as_str()) {
            // Root endpoint
            (HttpMethod::GET, "/") | (HttpMethod::GET, "/index.html") => {
                self.handle_index(&request)
            }

            // Health check endpoint
            (HttpMethod::GET, "/health") => self.handle_health(&request),

            // Echo endpoint - returns whatever is in the path
            (HttpMethod::GET, path) if path.starts_with("/echo/") => {
                self.handle_echo(&request, compression)
            }

            // User-agent endpoint - returns the User-Agent header
            (HttpMethod::GET, "/user-agent") => self.handle_user_agent(&request),

            // Files endpoints - GET and POST
            (HttpMethod::GET, path) if path.starts_with("/files/") => {
                self.handle_get_file(&request, compression)
            }
            (HttpMethod::POST, path) if path.starts_with("/files/") => {
                self.handle_post_file(&request)
            }
            (HttpMethod::DELETE, path) if path.starts_with("/files/") => {
                self.handle_delete_file(&request)
            }

            // API info endpoint
            (HttpMethod::GET, "/api/info") => self.handle_api_info(&request),

            // Headers endpoint - returns all request headers
            (HttpMethod::GET, "/headers") => self.handle_headers(&request, compression),

            // Default: 404 Not Found
            _ => Ok(HttpResponse::not_found()),
        }?;

        Ok(response.build())
    }

    /// Handle root endpoint
    fn handle_index(&self, _request: &HttpRequest) -> Result<HttpResponse> {
        Ok(HttpResponse::ok().html(
            r#"
<!DOCTYPE html>
<html>
<head>
    <title>Rust HTTP Server</title>
    <style>
        body { 
            font-family: 'Segoe UI', Arial, sans-serif; 
            max-width: 800px; 
            margin: 50px auto; 
            padding: 20px;
            background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
            color: white;
        }
        .container {
            background: rgba(255, 255, 255, 0.1);
            border-radius: 15px;
            padding: 30px;
            backdrop-filter: blur(10px);
        }
        h1 { margin-top: 0; }
        .feature { 
            background: rgba(255, 255, 255, 0.2); 
            padding: 15px; 
            margin: 10px 0; 
            border-radius: 8px;
            border-left: 4px solid #fff;
        }
        code { 
            background: rgba(0, 0, 0, 0.3); 
            padding: 2px 6px; 
            border-radius: 3px;
            font-family: 'Courier New', monospace;
        }
        .endpoint { margin: 8px 0; }
    </style>
</head>
<body>
    <div class="container">
        <h1>🦀 Production-Ready Rust HTTP Server</h1>
        <p><strong>Version 1.0.0</strong> - Built with best practices in mind</p>
        
        <div class="feature">
            <h3>✨ Features</h3>
            <ul>
                <li>✅ Concurrent request handling with thread pool</li>
                <li>✅ Multiple compression algorithms (Gzip, Deflate, Brotli)</li>
                <li>✅ Comprehensive error handling</li>
                <li>✅ Structured logging</li>
                <li>✅ File serving and uploads</li>
                <li>✅ RESTful API design</li>
            </ul>
        </div>
        
        <div class="feature">
            <h3>🔗 Available Endpoints</h3>
            <div class="endpoint">📍 <code>GET /</code> - This page</div>
            <div class="endpoint">📍 <code>GET /health</code> - Health check</div>
            <div class="endpoint">📍 <code>GET /echo/{text}</code> - Echo service</div>
            <div class="endpoint">📍 <code>GET /user-agent</code> - Get User-Agent header</div>
            <div class="endpoint">📍 <code>GET /files/{filename}</code> - Download file</div>
            <div class="endpoint">📍 <code>POST /files/{filename}</code> - Upload file</div>
            <div class="endpoint">📍 <code>DELETE /files/{filename}</code> - Delete file</div>
            <div class="endpoint">📍 <code>GET /headers</code> - View request headers</div>
            <div class="endpoint">📍 <code>GET /api/info</code> - Server information</div>
        </div>
    </div>
</body>
</html>
"#,
        ))
    }

    /// Handle health check endpoint
    fn handle_health(&self, _request: &HttpRequest) -> Result<HttpResponse> {
        let health = json!({
            "status": "healthy",
            "timestamp": chrono::Utc::now().to_rfc3339(),
            "uptime": "running"
        });

        HttpResponse::ok().json(&health)
    }

    /// Handle echo endpoint
    fn handle_echo(&self, request: &HttpRequest, compression: Compression) -> Result<HttpResponse> {
        let echo_str = &request.path[6..]; // Skip "/echo/"
        
        let response = HttpResponse::ok().text(echo_str);

        if compression != Compression::None {
            response.compress(compression)
        } else {
            Ok(response)
        }
    }

    /// Handle user-agent endpoint
    fn handle_user_agent(&self, request: &HttpRequest) -> Result<HttpResponse> {
        let user_agent = request
            .get_header("user-agent")
            .cloned()
            .unwrap_or_else(|| "Unknown".to_string());

        Ok(HttpResponse::ok().text(user_agent))
    }

    /// Handle GET file endpoint
    fn handle_get_file(&self, request: &HttpRequest, compression: Compression) -> Result<HttpResponse> {
        let filename = &request.path[7..]; // Skip "/files/"

        // Security: Prevent directory traversal
        if filename.contains("..") || filename.contains('/') || filename.contains('\\') {
            return Err(ServerError::InvalidRequest(
                "Invalid filename".to_string(),
            ));
        }

        let filepath = PathBuf::from(&self.file_directory).join(filename);

        let content = fs::read(&filepath).map_err(|_| {
            ServerError::FileNotFound(format!("File not found: {}", filename))
        })?;

        log::info!("Serving file: {} ({} bytes)", filename, content.len());

        let response = HttpResponse::ok()
            .header("Content-Type", Self::guess_content_type(filename))
            .body(content);

        if compression != Compression::None {
            response.compress(compression)
        } else {
            Ok(response)
        }
    }

    /// Handle POST file endpoint (file upload)
    fn handle_post_file(&self, request: &HttpRequest) -> Result<HttpResponse> {
        let filename = &request.path[7..]; // Skip "/files/"

        // Security: Prevent directory traversal
        if filename.contains("..") || filename.contains('/') || filename.contains('\\') {
            return Err(ServerError::InvalidRequest(
                "Invalid filename".to_string(),
            ));
        }

        let filepath = PathBuf::from(&self.file_directory).join(filename);

        // Ensure directory exists
        if let Some(parent) = filepath.parent() {
            fs::create_dir_all(parent)?;
        }

        fs::write(&filepath, &request.body)?;

        log::info!("File uploaded: {} ({} bytes)", filename, request.body.len());

        let response = json!({
            "message": "File uploaded successfully",
            "filename": filename,
            "size": request.body.len()
        });

        HttpResponse::created().json(&response)
    }

    /// Handle DELETE file endpoint
    fn handle_delete_file(&self, request: &HttpRequest) -> Result<HttpResponse> {
        let filename = &request.path[7..]; // Skip "/files/"

        // Security: Prevent directory traversal
        if filename.contains("..") || filename.contains('/') || filename.contains('\\') {
            return Err(ServerError::InvalidRequest(
                "Invalid filename".to_string(),
            ));
        }

        let filepath = PathBuf::from(&self.file_directory).join(filename);

        fs::remove_file(&filepath).map_err(|_| {
            ServerError::FileNotFound(format!("File not found: {}", filename))
        })?;

        log::info!("File deleted: {}", filename);

        let response = json!({
            "message": "File deleted successfully",
            "filename": filename
        });

        HttpResponse::ok().json(&response)
    }

    /// Handle API info endpoint
    fn handle_api_info(&self, _request: &HttpRequest) -> Result<HttpResponse> {
        let info = json!({
            "name": "Rust HTTP Server",
            "version": "1.0.0",
            "features": [
                "Concurrent connections",
                "HTTP compression (Gzip, Deflate, Brotli)",
                "File serving and uploads",
                "RESTful API design",
                "Comprehensive error handling",
                "Structured logging"
            ],
            "endpoints": {
                "GET": ["/", "/health", "/echo/{text}", "/user-agent", "/files/{filename}", "/headers", "/api/info"],
                "POST": ["/files/{filename}"],
                "DELETE": ["/files/{filename}"]
            }
        });

        HttpResponse::ok().json(&info)
    }

    /// Handle headers endpoint
    fn handle_headers(&self, request: &HttpRequest, compression: Compression) -> Result<HttpResponse> {
        let headers_json = json!(request.headers);
        let response = HttpResponse::ok().json(&headers_json)?;

        if compression != Compression::None {
            response.compress(compression)
        } else {
            Ok(response)
        }
    }

    /// Guess content type from file extension
    fn guess_content_type(filename: &str) -> &'static str {
        let ext = Path::new(filename)
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("");

        match ext {
            "html" | "htm" => "text/html",
            "css" => "text/css",
            "js" => "application/javascript",
            "json" => "application/json",
            "png" => "image/png",
            "jpg" | "jpeg" => "image/jpeg",
            "gif" => "image/gif",
            "svg" => "image/svg+xml",
            "txt" => "text/plain",
            "pdf" => "application/pdf",
            "zip" => "application/zip",
            _ => "application/octet-stream",
        }
    }
}
