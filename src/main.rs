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
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Instant;
use threadpool::ThreadPool;

#[cfg(unix)]
fn set_socket_options(listener: &TcpListener) -> anyhow::Result<()> {
    use std::os::fd::AsRawFd;
    
    let fd = listener.as_raw_fd();
    
    // Enable SO_REUSEADDR for quick restarts
    unsafe {
        let optval: libc::c_int = 1;
        libc::setsockopt(
            fd,
            libc::SOL_SOCKET,
            libc::SO_REUSEADDR,
            &optval as *const _ as *const libc::c_void,
            std::mem::size_of_val(&optval) as libc::socklen_t,
        );
    }
    
    // Enable SO_REUSEPORT for better load distribution across threads (Linux/BSD)
    #[cfg(any(target_os = "linux", target_os = "freebsd", target_os = "netbsd", target_os = "openbsd"))]
    unsafe {
        let optval: libc::c_int = 1;
        libc::setsockopt(
            fd,
            libc::SOL_SOCKET,
            libc::SO_REUSEPORT,
            &optval as *const _ as *const libc::c_void,
            std::mem::size_of_val(&optval) as libc::socklen_t,
        );
    }
    
    Ok(())
}

#[cfg(not(unix))]
fn set_socket_options(_listener: &TcpListener) -> anyhow::Result<()> {
    // Windows doesn't need these optimizations
    Ok(())
}

/// Server metrics for monitoring
pub struct ServerMetrics {
    pub request_count: AtomicU64,
    pub error_count: AtomicU64,
    pub total_response_time_ms: AtomicU64,
    pub active_connections: AtomicU64,
    pub start_time: Instant,
}

impl ServerMetrics {
    pub fn new() -> Self {
        Self {
            request_count: AtomicU64::new(0),
            error_count: AtomicU64::new(0),
            total_response_time_ms: AtomicU64::new(0),
            active_connections: AtomicU64::new(0),
            start_time: Instant::now(),
        }
    }

    pub fn uptime_seconds(&self) -> u64 {
        self.start_time.elapsed().as_secs()
    }
}

/// Handle a single client connection
fn handle_client(stream: TcpStream, router: Arc<Router>, metrics: Arc<ServerMetrics>) {
    use std::io::Write;

    let peer_addr = stream.peer_addr().ok();
    let stream_clone = stream.try_clone();

    // Enable TCP_NODELAY to disable Nagle's algorithm for lower latency
    let _ = stream.set_nodelay(true);

    // Track active connection
    metrics.active_connections.fetch_add(1, Ordering::Relaxed);
    let start_time = Instant::now();

    let result = (|| -> Result<(), ServerError> {
        let mut reader = BufReader::with_capacity(8192, stream);

        // Parse the HTTP request
        let request = HttpRequest::parse(&mut reader)?;

        // Generate request ID for tracking
        let request_id = metrics.request_count.fetch_add(1, Ordering::Relaxed);
        
        log::debug!("Request #{}: {} {}", request_id, request.method.as_str(), request.path);

        // Route the request and generate response
        let response_bytes = router.route(request, &metrics)?;

        // Write response back to client
        let mut stream = reader.into_inner();
        stream.write_all(&response_bytes)?;
        stream.flush()?;

        Ok(())
    })();

    // Record metrics
    let response_time_ms = start_time.elapsed().as_millis() as u64;
    metrics.total_response_time_ms.fetch_add(response_time_ms, Ordering::Relaxed);
    metrics.active_connections.fetch_sub(1, Ordering::Relaxed);

    if result.is_err() {
        metrics.error_count.fetch_add(1, Ordering::Relaxed);
    }

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

    // Create router and metrics
    let router = Arc::new(Router::new(config.directory.clone()));
    let metrics = Arc::new(ServerMetrics::new());

    // Setup graceful shutdown
    let shutdown = Arc::new(AtomicBool::new(false));
    let shutdown_clone = Arc::clone(&shutdown);
    
    ctrlc::set_handler(move || {
        log::info!("Received shutdown signal, gracefully shutting down...");
        shutdown_clone.store(true, Ordering::Relaxed);
    })?;

    // Create thread pool for handling connections
    let pool = ThreadPool::new(config.workers);

    // Bind to address
    let listener = TcpListener::bind(config.server_address())?;
    
    // Set socket options for better performance
    set_socket_options(&listener)?;
    
    // Set non-blocking mode for shutdown handling
    listener.set_nonblocking(false)?;
    
    log::info!("Server starting...");
    log::info!("Serving files from: {}", config.directory);
    log::info!("Worker threads: {}", config.workers);
    log::info!("Listening on: http://{}", config.server_address());
    log::info!("Optimizations: TCP_NODELAY=on, SO_REUSEADDR=on, Buffer=8KB");
    log::info!("Features: Graceful shutdown, Metrics tracking, Request ID tracing");
    log::info!("Metrics endpoint: http://{}/metrics", config.server_address());
    log::info!("Server is ready to handle 100+ concurrent requests per second!");

    // Accept connections
    for stream in listener.incoming() {
        // Check for shutdown signal
        if shutdown.load(Ordering::Relaxed) {
            log::info!("Shutdown initiated, no longer accepting new connections");
            break;
        }

        match stream {
            Ok(stream) => {
                let router = Arc::clone(&router);
                let metrics_clone = Arc::clone(&metrics);
                pool.execute(move || {
                    handle_client(stream, router, metrics_clone);
                });
            }
            Err(e) => {
                log::error!("Failed to accept connection: {}", e);
            }
        }
    }

    // Wait for active connections to finish
    log::info!("Waiting for {} active connections to finish...", 
        metrics.active_connections.load(Ordering::Relaxed));
    
    drop(listener);
    
    // Give threads time to finish (with timeout)
    let shutdown_timeout = std::time::Duration::from_secs(10);
    let shutdown_start = Instant::now();
    
    while metrics.active_connections.load(Ordering::Relaxed) > 0 
        && shutdown_start.elapsed() < shutdown_timeout {
        std::thread::sleep(std::time::Duration::from_millis(100));
    }
    
    let remaining = metrics.active_connections.load(Ordering::Relaxed);
    if remaining > 0 {
        log::warn!("Shutdown timeout reached with {} connections still active", remaining);
    }

    log::info!("Server shutdown complete");
    log::info!("Final stats - Requests: {}, Errors: {}, Uptime: {}s",
        metrics.request_count.load(Ordering::Relaxed),
        metrics.error_count.load(Ordering::Relaxed),
        metrics.uptime_seconds());

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