use crate::error::{Result, ServerError};
use std::collections::HashMap;
use std::io::{BufRead, BufReader, Read};

/// HTTP methods supported by the server
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HttpMethod {
    GET,
    POST,
    PUT,
    DELETE,
    PATCH,
    HEAD,
    OPTIONS,
}

impl HttpMethod {
    pub fn from_str(s: &str) -> Result<Self> {
        match s.to_uppercase().as_str() {
            "GET" => Ok(HttpMethod::GET),
            "POST" => Ok(HttpMethod::POST),
            "PUT" => Ok(HttpMethod::PUT),
            "DELETE" => Ok(HttpMethod::DELETE),
            "PATCH" => Ok(HttpMethod::PATCH),
            "HEAD" => Ok(HttpMethod::HEAD),
            "OPTIONS" => Ok(HttpMethod::OPTIONS),
            _ => Err(ServerError::InvalidMethod(s.to_string())),
        }
    }

    pub fn as_str(&self) -> &str {
        match self {
            HttpMethod::GET => "GET",
            HttpMethod::POST => "POST",
            HttpMethod::PUT => "PUT",
            HttpMethod::DELETE => "DELETE",
            HttpMethod::PATCH => "PATCH",
            HttpMethod::HEAD => "HEAD",
            HttpMethod::OPTIONS => "OPTIONS",
        }
    }
}

/// Represents an HTTP request
#[derive(Debug)]
pub struct HttpRequest {
    pub method: HttpMethod,
    pub path: String,
    pub version: String,
    pub headers: HashMap<String, String>,
    pub body: Vec<u8>,
}

impl HttpRequest {
    /// Parse an HTTP request from a TCP stream
    pub fn parse<R: Read>(reader: &mut BufReader<R>) -> Result<Self> {
        // Parse request line
        let mut request_line = String::new();
        reader
            .read_line(&mut request_line)
            .map_err(|e| ServerError::InvalidRequest(format!("Failed to read request line: {}", e)))?;

        let parts: Vec<&str> = request_line.trim().split_whitespace().collect();
        if parts.len() < 3 {
            return Err(ServerError::InvalidRequest(
                "Invalid request line format".to_string(),
            ));
        }

        let method = HttpMethod::from_str(parts[0])?;
        let path = parts[1].to_string();
        let version = parts[2].to_string();

        // Parse headers
        let mut headers = HashMap::new();
        let mut content_length = 0usize;

        for line in reader.by_ref().lines() {
            let line = line.map_err(|e| {
                ServerError::InvalidRequest(format!("Failed to read header line: {}", e))
            })?;

            if line.is_empty() {
                break;
            }

            if let Some((key, value)) = line.split_once(':') {
                let key = key.trim().to_lowercase();
                let value = value.trim().to_string();

                if key == "content-length" {
                    content_length = value.parse().unwrap_or(0);
                }

                headers.insert(key, value);
            }
        }

        // Read body if present
        let mut body = vec![0u8; content_length];
        if content_length > 0 {
            reader.read_exact(&mut body).map_err(|e| {
                ServerError::InvalidRequest(format!("Failed to read request body: {}", e))
            })?;
        }

        Ok(HttpRequest {
            method,
            path,
            version,
            headers,
            body,
        })
    }

    /// Get a header value (case-insensitive)
    pub fn get_header(&self, key: &str) -> Option<&String> {
        self.headers.get(&key.to_lowercase())
    }

    /// Get accepted encoding from Accept-Encoding header
    pub fn get_accepted_encodings(&self) -> Vec<String> {
        self.get_header("accept-encoding")
            .map(|value| {
                value
                    .split(',')
                    .map(|s| s.trim().to_lowercase())
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Get request body as string
    pub fn body_as_string(&self) -> Result<String> {
        String::from_utf8(self.body.clone())
            .map_err(|e| ServerError::ParseError(format!("Invalid UTF-8 in body: {}", e)))
    }

    /// Check if request accepts a specific encoding
    pub fn accepts_encoding(&self, encoding: &str) -> bool {
        self.get_accepted_encodings()
            .iter()
            .any(|e| e == encoding || e == "*")
    }
}
