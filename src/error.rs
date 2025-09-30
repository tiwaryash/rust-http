use std::io;
use thiserror::Error;

/// Custom error types for the HTTP server
#[derive(Error, Debug)]
pub enum ServerError {
    #[error("IO error: {0}")]
    Io(#[from] io::Error),

    #[error("Invalid HTTP request: {0}")]
    InvalidRequest(String),

    #[error("Invalid HTTP method: {0}")]
    InvalidMethod(String),

    #[error("File not found: {0}")]
    FileNotFound(String),

    #[error("Compression error: {0}")]
    CompressionError(String),

    #[error("Parse error: {0}")]
    ParseError(String),

    #[error("Configuration error: {0}")]
    ConfigError(String),

    #[error("Internal server error: {0}")]
    InternalError(String),
}

pub type Result<T> = std::result::Result<T, ServerError>;

impl ServerError {
    /// Convert error to HTTP status code
    pub fn status_code(&self) -> u16 {
        match self {
            ServerError::FileNotFound(_) => 404,
            ServerError::InvalidRequest(_) | ServerError::InvalidMethod(_) => 400,
            ServerError::ParseError(_) => 400,
            _ => 500,
        }
    }

    /// Convert error to HTTP response
    pub fn to_response(&self) -> String {
        let status_code = self.status_code();
        let status_text = match status_code {
            400 => "Bad Request",
            404 => "Not Found",
            500 => "Internal Server Error",
            _ => "Error",
        };

        format!(
            "HTTP/1.1 {} {}\r\nContent-Type: text/plain\r\nContent-Length: {}\r\n\r\n{}",
            status_code,
            status_text,
            self.to_string().len(),
            self.to_string()
        )
    }
}
