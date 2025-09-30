# 🦀 Production-Ready HTTP Server in Rust

A high-performance, feature-rich HTTP server implementation in Rust, demonstrating industry best practices for backend development.

![Rust](https://img.shields.io/badge/rust-%23000000.svg?style=for-the-badge&logo=rust&logoColor=white)
![License](https://img.shields.io/badge/license-MIT-blue.svg)

## ✨ Features

### Core HTTP Functionality
- ✅ **Port Binding** - Configurable host and port binding
- ✅ **HTTP/1.1 Protocol** - Full HTTP/1.1 support with proper status codes
- ✅ **URL Path Extraction** - Robust request parsing and routing
- ✅ **Request/Response Body Handling** - Support for reading and writing request bodies
- ✅ **Header Management** - Complete header parsing and manipulation
- ✅ **Concurrent Connections** - Thread pool-based architecture for handling multiple connections
- ✅ **File Operations** - Serve, upload, and delete files with proper content-type detection

### Advanced Features
- 🗜️ **HTTP Compression** - Support for Gzip, Deflate, and Brotli compression
- 🔒 **Security** - Path traversal protection and input validation
- 📊 **Structured Logging** - Comprehensive logging with configurable verbosity
- ⚡ **High Performance** - Efficient thread pool management
- 🎯 **RESTful API Design** - Clean, intuitive API endpoints
- 🛡️ **Error Handling** - Custom error types with proper HTTP status code mapping
- 🔧 **Configuration** - CLI arguments and environment variable support
- 📝 **Content Type Detection** - Automatic MIME type detection for files

## 🏗️ Architecture

The server follows a modular architecture with clear separation of concerns:

```
src/
├── main.rs           # Application entry point and connection handling
├── config.rs         # Configuration management
├── error.rs          # Custom error types
├── request.rs        # HTTP request parsing
├── response.rs       # HTTP response building
├── compression.rs    # Compression algorithms
└── router.rs         # Request routing and handlers
```

### Design Patterns

- **Builder Pattern** - Used in `HttpResponse` for flexible response construction
- **Error Handling** - Custom error types with `thiserror` for ergonomic error handling
- **Thread Pool Pattern** - Efficient concurrent request handling
- **Modular Design** - Clean separation of concerns with well-defined modules

## 🚀 Quick Start

### Prerequisites

- Rust 1.80+ (Install from [rustup.rs](https://rustup.rs/))

### Installation

```bash
# Clone the repository
git clone https://github.com/tiwaryash/rust-http.git
cd rust-http

# Build the project
cargo build --release

# Run the server
cargo run --release
```

### Basic Usage

```bash
# Start server on default port (4221)
cargo run --release

# Custom configuration
cargo run --release -- --port 8080 --host 0.0.0.0 --directory ./files --workers 8

# With verbose logging
cargo run --release -- --verbose

# Using environment variables
HTTP_PORT=8080 FILE_DIRECTORY=./uploads cargo run --release
```

## 📚 API Documentation

### Endpoints

#### Health & Information

| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/` | Welcome page with HTML interface |
| GET | `/health` | Health check endpoint (returns JSON) |
| GET | `/api/info` | Server information and available endpoints |

#### Utility Endpoints

| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/echo/{text}` | Echo back the text from URL path |
| GET | `/user-agent` | Return the User-Agent header |
| GET | `/headers` | Return all request headers as JSON |

#### File Operations

| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/files/{filename}` | Download a file |
| POST | `/files/{filename}` | Upload a file |
| DELETE | `/files/{filename}` | Delete a file |

### Example Requests

#### Echo Service (with compression)
```bash
curl -H "Accept-Encoding: gzip" http://localhost:4221/echo/HelloWorld
```

#### Get User Agent
```bash
curl http://localhost:4221/user-agent
```

#### Upload a File
```bash
curl -X POST -d "Hello, World!" http://localhost:4221/files/hello.txt
```

#### Download a File
```bash
curl http://localhost:4221/files/hello.txt
```

#### Get Server Info
```bash
curl http://localhost:4221/api/info
```

#### Health Check
```bash
curl http://localhost:4221/health
```

## 🔧 Configuration

### Command Line Arguments

| Argument | Short | Default | Description |
|----------|-------|---------|-------------|
| `--port` | `-p` | 4221 | Port to bind to |
| `--host` | | 127.0.0.1 | Host address to bind to |
| `--directory` | `-d` | . | Directory for file operations |
| `--workers` | `-w` | 4 | Number of worker threads |
| `--verbose` | `-v` | false | Enable verbose logging |

### Environment Variables

| Variable | Default | Description |
|----------|---------|-------------|
| `HTTP_PORT` | 4221 | Server port |
| `HTTP_HOST` | 127.0.0.1 | Server host |
| `FILE_DIRECTORY` | . | File serving directory |
| `WORKER_THREADS` | 4 | Thread pool size |
| `RUST_LOG` | info | Log level (trace, debug, info, warn, error) |

## 🗜️ Compression Support

The server automatically compresses responses based on the `Accept-Encoding` header:

- **Gzip** - Most widely supported
- **Deflate** - Standard compression
- **Brotli** - Modern, highly efficient compression

Compression is applied to:
- Echo endpoint responses
- File downloads (when requested)
- Headers endpoint responses
- All responses > 100 bytes

## 🛡️ Security Features

- **Path Traversal Protection** - Prevents access to files outside the configured directory
- **Input Validation** - All inputs are validated before processing
- **Error Information Hiding** - Production-ready error messages that don't leak sensitive information
- **Safe File Operations** - Proper error handling for all file operations

## 📊 Logging

The server uses structured logging with different levels:

```bash
# Info level (default)
cargo run --release

# Debug level
cargo run --release -- --verbose

# Custom log level
RUST_LOG=debug cargo run --release
```

Log format includes:
- Timestamp (milliseconds precision)
- Log level
- Message
- Request method, path, and body size

## 🧪 Testing

Run the test suite:

```bash
# Run all tests
cargo test

# Run with output
cargo test -- --nocapture

# Run specific test
cargo test test_gzip_compression
```

Test the server manually:

```bash
# Terminal 1: Start server
cargo run --release -- --verbose

# Terminal 2: Run tests
curl http://localhost:4221/
curl http://localhost:4221/health
curl -H "Accept-Encoding: gzip" http://localhost:4221/echo/test
curl -X POST -d "test data" http://localhost:4221/files/test.txt
curl http://localhost:4221/files/test.txt
```

## 📈 Performance Characteristics

- **Concurrent Connections**: Thread pool-based handling (configurable)
- **Compression Ratio**: 60-80% reduction for text content
- **Memory Usage**: Efficient buffer management with `bytes` crate
- **Request Handling**: Non-blocking connection acceptance

## 🎯 Best Practices Demonstrated

### Code Quality
- ✅ Modular architecture with clear separation of concerns
- ✅ Comprehensive error handling with custom error types
- ✅ Rust idioms: ownership, borrowing, and lifetimes
- ✅ Type safety and zero-cost abstractions

### Backend Development
- ✅ RESTful API design
- ✅ Proper HTTP status codes
- ✅ Content-Type negotiation
- ✅ Compression support
- ✅ Structured logging
- ✅ Configuration management

### Production Readiness
- ✅ Error handling and recovery
- ✅ Security considerations
- ✅ Performance optimization
- ✅ Comprehensive documentation
- ✅ Testing infrastructure

## 🔄 Development

### Build from source
```bash
cargo build
```

### Run in development mode
```bash
cargo run -- --verbose --directory ./test_files
```

### Format code
```bash
cargo fmt
```

### Run linter
```bash
cargo clippy -- -D warnings
```

## 📦 Dependencies

| Crate | Purpose |
|-------|---------|
| `anyhow` | Error handling |
| `thiserror` | Custom error types |
| `bytes` | Buffer management |
| `flate2` | Gzip/Deflate compression |
| `brotli` | Brotli compression |
| `serde` | Serialization framework |
| `serde_json` | JSON support |
| `log` | Logging facade |
| `env_logger` | Logging implementation |
| `clap` | CLI argument parsing |
| `chrono` | Date/time handling |
| `threadpool` | Thread pool management |
| `regex` | Pattern matching |

## 🤝 Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## 📄 License

This project is licensed under the MIT License - see the LICENSE file for details.

## 👨‍💻 Author

Built with ❤️ by [Yash Tiwari](https://github.com/tiwaryash)

## 🎓 Learning Resources

This project demonstrates:
- HTTP protocol implementation
- Rust systems programming
- Concurrent programming patterns
- Error handling strategies
- API design principles
- Production-ready code practices

## 🌟 Acknowledgments

- Built as part of the CodeCrafters HTTP Server challenge
- Inspired by production HTTP servers like nginx and Apache
- Uses industry-standard Rust crates and patterns

---

**Note**: This server is designed for educational and portfolio purposes. For production use cases, consider battle-tested solutions like Actix-web, Rocket, or Axum.