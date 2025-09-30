use clap::Parser;
use std::path::PathBuf;

/// Configuration for the HTTP server
#[derive(Parser, Debug, Clone)]
#[command(
    name = "http-server",
    about = "A production-ready HTTP server written in Rust",
    version = "1.0.0"
)]
pub struct Config {
    /// Port to bind the server to
    #[arg(short, long, default_value = "4221", env = "HTTP_PORT")]
    pub port: u16,

    /// Host address to bind to
    #[arg(long, default_value = "127.0.0.1", env = "HTTP_HOST")]
    pub host: String,

    /// Directory to serve files from
    #[arg(short, long, default_value = ".", env = "FILE_DIRECTORY")]
    pub directory: String,

    /// Number of worker threads for handling connections
    #[arg(short, long, default_value = "4", env = "WORKER_THREADS")]
    pub workers: usize,

    /// Enable verbose logging
    #[arg(short, long, default_value = "false")]
    pub verbose: bool,
}

impl Config {
    /// Parse configuration from command line arguments and environment variables
    pub fn parse_config() -> Self {
        Config::parse()
    }

    /// Get the full server address (host:port)
    pub fn server_address(&self) -> String {
        format!("{}:{}", self.host, self.port)
    }

    /// Validate the configuration
    pub fn validate(&self) -> Result<(), String> {
        // Validate port
        if self.port == 0 {
            return Err("Port must be greater than 0".to_string());
        }

        // Validate directory
        let path = PathBuf::from(&self.directory);
        if !path.exists() {
            log::warn!(
                "Directory '{}' does not exist, will be created on first file upload",
                self.directory
            );
        }

        // Validate worker threads
        if self.workers == 0 {
            return Err("Number of workers must be greater than 0".to_string());
        }

        Ok(())
    }

    /// Initialize logger based on configuration
    pub fn init_logger(&self) {
        let log_level = if self.verbose {
            "debug"
        } else {
            "info"
        };

        env_logger::Builder::from_env(env_logger::Env::default().default_filter_or(log_level))
            .format_timestamp_millis()
            .init();
    }
}
