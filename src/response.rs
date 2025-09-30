use crate::compression::Compression;
use crate::error::Result;
use std::collections::HashMap;

/// HTTP response builder
#[derive(Debug)]
pub struct HttpResponse {
    status_code: u16,
    status_text: String,
    headers: HashMap<String, String>,
    body: Vec<u8>,
}

impl HttpResponse {
    /// Create a new response with status code
    pub fn new(status_code: u16) -> Self {
        let status_text = Self::status_text(status_code);
        HttpResponse {
            status_code,
            status_text,
            headers: HashMap::new(),
            body: Vec::new(),
        }
    }

    /// Get standard status text for a status code
    fn status_text(code: u16) -> String {
        match code {
            200 => "OK",
            201 => "Created",
            204 => "No Content",
            400 => "Bad Request",
            404 => "Not Found",
            405 => "Method Not Allowed",
            500 => "Internal Server Error",
            _ => "Unknown",
        }
        .to_string()
    }

    /// Set a header
    pub fn header(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.headers.insert(key.into(), value.into());
        self
    }

    /// Set the response body
    pub fn body(mut self, body: impl Into<Vec<u8>>) -> Self {
        self.body = body.into();
        self
    }

    /// Set the response body as text
    pub fn text(self, text: impl Into<String>) -> Self {
        self.header("Content-Type", "text/plain")
            .body(text.into().into_bytes())
    }

    /// Set the response body as JSON
    pub fn json(self, json: &impl serde::Serialize) -> Result<Self> {
        let json_str = serde_json::to_string(json)
            .map_err(|e| crate::error::ServerError::InternalError(format!("JSON error: {}", e)))?;
        Ok(self
            .header("Content-Type", "application/json")
            .body(json_str.into_bytes()))
    }

    /// Set the response body as HTML
    pub fn html(self, html: impl Into<String>) -> Self {
        self.header("Content-Type", "text/html")
            .body(html.into().into_bytes())
    }

    /// Apply compression to the response body
    pub fn compress(mut self, compression: Compression) -> Result<Self> {
        if self.body.is_empty() {
            return Ok(self);
        }

        let compressed = compression.compress(&self.body)?;
        self.body = compressed;
        self.headers
            .insert("Content-Encoding".to_string(), compression.name().to_string());
        Ok(self)
    }

    /// Build the HTTP response as bytes
    pub fn build(mut self) -> Vec<u8> {
        // Set Content-Length if not already set
        if !self.headers.contains_key("Content-Length") {
            self.headers
                .insert("Content-Length".to_string(), self.body.len().to_string());
        }

        // Build response
        let mut response = format!(
            "HTTP/1.1 {} {}\r\n",
            self.status_code, self.status_text
        );

        for (key, value) in &self.headers {
            response.push_str(&format!("{}: {}\r\n", key, value));
        }

        response.push_str("\r\n");

        let mut bytes = response.into_bytes();
        bytes.extend_from_slice(&self.body);
        bytes
    }
}

// Convenient constructors
impl HttpResponse {
    pub fn ok() -> Self {
        Self::new(200)
    }

    pub fn created() -> Self {
        Self::new(201)
    }

    pub fn no_content() -> Self {
        Self::new(204)
    }

    pub fn bad_request() -> Self {
        Self::new(400)
    }

    pub fn not_found() -> Self {
        Self::new(404).text("404 - Not Found")
    }

    pub fn method_not_allowed() -> Self {
        Self::new(405).text("405 - Method Not Allowed")
    }

    pub fn internal_error() -> Self {
        Self::new(500).text("500 - Internal Server Error")
    }
}
