use crate::error::{Result, ServerError};
use brotli::enc::BrotliEncoderParams;
use flate2::write::{DeflateEncoder, GzEncoder};
use flate2::Compression as FlateCompression;
use std::io::Write;

/// Compression algorithms supported by the server
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Compression {
    Gzip,
    Deflate,
    Brotli,
    None,
}

impl Compression {
    /// Get compression from Accept-Encoding header value
    pub fn from_accept_encoding(encodings: &[String]) -> Self {
        for encoding in encodings {
            match encoding.as_str() {
                "br" => return Compression::Brotli,
                "gzip" => return Compression::Gzip,
                "deflate" => return Compression::Deflate,
                _ => continue,
            }
        }
        Compression::None
    }

    /// Get the name of the compression algorithm
    pub fn name(&self) -> &str {
        match self {
            Compression::Gzip => "gzip",
            Compression::Deflate => "deflate",
            Compression::Brotli => "br",
            Compression::None => "identity",
        }
    }

    /// Compress data using the selected algorithm
    pub fn compress(&self, data: &[u8]) -> Result<Vec<u8>> {
        match self {
            Compression::Gzip => Self::gzip_compress(data),
            Compression::Deflate => Self::deflate_compress(data),
            Compression::Brotli => Self::brotli_compress(data),
            Compression::None => Ok(data.to_vec()),
        }
    }

    /// Compress data using gzip
    fn gzip_compress(data: &[u8]) -> Result<Vec<u8>> {
        let mut encoder = GzEncoder::new(Vec::new(), FlateCompression::default());
        encoder
            .write_all(data)
            .map_err(|e| ServerError::CompressionError(format!("Gzip compression failed: {}", e)))?;
        encoder
            .finish()
            .map_err(|e| ServerError::CompressionError(format!("Gzip finish failed: {}", e)))
    }

    /// Compress data using deflate
    fn deflate_compress(data: &[u8]) -> Result<Vec<u8>> {
        let mut encoder = DeflateEncoder::new(Vec::new(), FlateCompression::default());
        encoder.write_all(data).map_err(|e| {
            ServerError::CompressionError(format!("Deflate compression failed: {}", e))
        })?;
        encoder
            .finish()
            .map_err(|e| ServerError::CompressionError(format!("Deflate finish failed: {}", e)))
    }

    /// Compress data using brotli
    fn brotli_compress(data: &[u8]) -> Result<Vec<u8>> {
        let mut output = Vec::new();
        let params = BrotliEncoderParams::default();

        brotli::BrotliCompress(
            &mut std::io::Cursor::new(data),
            &mut output,
            &params,
        )
        .map_err(|e| {
            ServerError::CompressionError(format!("Brotli compression failed: {}", e))
        })?;

        Ok(output)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gzip_compression() {
        let data = b"Hello, World! This is a test string for compression.";
        let compressed = Compression::Gzip.compress(data).unwrap();
        assert!(compressed.len() < data.len());
    }

    #[test]
    fn test_deflate_compression() {
        let data = b"Hello, World! This is a test string for compression.";
        let compressed = Compression::Deflate.compress(data).unwrap();
        assert!(compressed.len() < data.len());
    }

    #[test]
    fn test_brotli_compression() {
        let data = b"Hello, World! This is a test string for compression.";
        let compressed = Compression::Brotli.compress(data).unwrap();
        assert!(compressed.len() < data.len());
    }

    #[test]
    fn test_from_accept_encoding() {
        let encodings = vec!["gzip".to_string(), "deflate".to_string()];
        assert_eq!(Compression::from_accept_encoding(&encodings), Compression::Gzip);

        let encodings = vec!["br".to_string()];
        assert_eq!(Compression::from_accept_encoding(&encodings), Compression::Brotli);

        let encodings = vec!["identity".to_string()];
        assert_eq!(Compression::from_accept_encoding(&encodings), Compression::None);
    }
}
