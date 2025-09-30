mod compression;
mod config;
mod error;
mod request;
mod response;
mod router;

use config::Config;
use error::ServerError;
use request::HttpRequest;
use router::Router;
use std::io::BufReader;
use std::net::{TcpListener, TcpStream};
use std::sync::Arc;
use threadpool::ThreadPool;

/// Handle a single client connection
fn handle_client(stream: TcpStream, router: Arc<Router>) {
    use std::io::Write;

    let peer_addr = stream.peer_addr().ok();
    let stream_clone = stream.try_clone();

    let result = (|| -> Result<(), ServerError> {
        let mut reader = BufReader::new(stream);

        // Parse the HTTP request
        let request = HttpRequest::parse(&mut reader)?;

        // Route the request and generate response
        let response_bytes = router.route(request)?;

        // Write response back to client
        let mut stream = reader.into_inner();
        stream.write_all(&response_bytes)?;
        stream.flush()?;

        Ok(())
    })();

    // Log errors if any
    if let Err(e) = result {
        log::error!(
            "Error handling request from {:?}: {}",
            peer_addr.unwrap_or_else(|| "unknown".parse().unwrap()),
            e
        );

        // Try to send error response using cloned stream
        if let Ok(mut stream_for_error) = stream_clone {
            let error_response = e.to_response();
            let _ = stream_for_error.write_all(error_response.as_bytes());
            let _ = stream_for_error.flush();
        }
    }
}

fn main() -> anyhow::Result<()> {
    // Parse configuration
    let config = Config::parse_config();

    // Initialize logger
    config.init_logger();

    // Validate configuration
    if let Err(e) = config.validate() {
        log::error!("Configuration error: {}", e);
        std::process::exit(1);
    }

    // Create router
    let router = Arc::new(Router::new(config.directory.clone()));

    // Create thread pool for handling connections
    let pool = ThreadPool::new(config.workers);

    // Bind to address
    let listener = TcpListener::bind(config.server_address())?;

    log::info!("Server starting...");
    log::info!("Serving files from: {}", config.directory);
    log::info!("Worker threads: {}", config.workers);
    log::info!("Listening on: http://{}", config.server_address());
    log::info!("Server is ready to accept connections!");

    // Accept connections
    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                let router = Arc::clone(&router);
                pool.execute(move || {
                    handle_client(stream, router);
                });
            }
            Err(e) => {
                log::error!("Failed to accept connection: {}", e);
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_server_configuration() {
        let config = Config {
            port: 8080,
            host: "127.0.0.1".to_string(),
            directory: ".".to_string(),
            workers: 4,
            verbose: false,
        };

        assert_eq!(config.server_address(), "127.0.0.1:8080");
        assert!(config.validate().is_ok());
    }
}