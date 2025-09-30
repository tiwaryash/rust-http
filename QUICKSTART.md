# 🚀 Quick Start Guide

## Installation & Running

```bash
# Build the server
cargo build --release

# Run with default settings (port 4221)
cargo run --release

# Run with custom configuration
cargo run --release -- --port 8080 --directory ./files --workers 8 --verbose

# Using environment variables
HTTP_PORT=8080 HTTP_HOST=0.0.0.0 FILE_DIRECTORY=./uploads cargo run --release
```

## Testing All Features

### 1. Start the Server
```bash
# Terminal 1
cargo run --release -- --verbose --directory ./test_files
```

### 2. Run Automated Tests
```bash
# Terminal 2
./run_tests.sh
```

### 3. Manual Testing

#### Basic Endpoints
```bash
# Welcome page (HTML)
curl http://localhost:4221/

# Health check
curl http://localhost:4221/health

# Echo service
curl http://localhost:4221/echo/HelloWorld

# Get User-Agent
curl http://localhost:4221/user-agent

# View all headers
curl http://localhost:4221/headers

# Server info
curl http://localhost:4221/api/info | jq
```

#### File Operations
```bash
# Upload a file
echo "Hello, World!" | curl -X POST --data-binary @- http://localhost:4221/files/hello.txt

# Download a file
curl http://localhost:4221/files/hello.txt

# Delete a file
curl -X DELETE http://localhost:4221/files/hello.txt
```

#### Compression Testing
```bash
# Gzip compression
curl -H "Accept-Encoding: gzip" --compressed http://localhost:4221/echo/TestCompressionWithGzip

# Deflate compression
curl -H "Accept-Encoding: deflate" http://localhost:4221/echo/TestDeflateCompression

# Brotli compression (best)
curl -H "Accept-Encoding: br, gzip, deflate" http://localhost:4221/echo/BrotliIsBest

# View compression header
curl -i -H "Accept-Encoding: gzip" http://localhost:4221/echo/test | grep Content-Encoding
```

#### Concurrent Connections
```bash
# Send 20 concurrent requests
for i in {1..20}; do
  curl -s http://localhost:4221/echo/request-$i &
done
wait
```

## Architecture Overview

```
┌─────────────────────────────────────────────────────┐
│                    main.rs                           │
│  (Entry point, Thread Pool, Connection Handler)     │
└────────────────────┬────────────────────────────────┘
                     │
        ┌────────────┴────────────┬─────────────┬──────────────┐
        ▼                         ▼             ▼              ▼
   ┌─────────┐              ┌──────────┐  ┌─────────┐   ┌─────────┐
   │ config  │              │ request  │  │response │   │  error  │
   │.rs      │              │.rs       │  │.rs      │   │.rs      │
   └─────────┘              └──────────┘  └─────────┘   └─────────┘
Configuration              HTTP Request   HTTP Response  Custom
Management                 Parsing        Building       Error Types
        │                         │             │              │
        └────────────┬────────────┴─────────────┴──────────────┘
                     ▼
            ┌────────────────┐
            │    router.rs   │
            │  (Request      │
            │   Routing)     │
            └────────┬───────┘
                     │
            ┌────────┴────────┐
            ▼                 ▼
      ┌──────────┐      ┌──────────┐
      │compression│      │File Ops  │
      │.rs        │      │Handler   │
      └───────────┘      └──────────┘
```

## Key Features Checklist

### ✅ Core HTTP Features
- [x] Bind to configurable port
- [x] Respond with 200 OK
- [x] Extract URL path
- [x] Respond with body
- [x] Read headers
- [x] Concurrent connections (Thread Pool)
- [x] Return files
- [x] Read request body
- [x] POST file uploads
- [x] DELETE files

### ✅ Advanced Features
- [x] HTTP Compression (Gzip, Deflate, Brotli)
- [x] Compression header negotiation
- [x] Multiple compression schemes
- [x] JSON API responses
- [x] HTML responses
- [x] Content-Type detection
- [x] Security (path traversal protection)
- [x] Structured logging
- [x] Error handling
- [x] Configuration management

## Performance Tips

### Optimize Worker Threads
```bash
# For CPU-intensive: cores * 2
cargo run --release -- --workers 8

# For I/O-intensive: cores * 4
cargo run --release -- --workers 16
```

### Enable Compression
Always send `Accept-Encoding` header for better performance:
```bash
curl -H "Accept-Encoding: br, gzip, deflate" http://localhost:4221/...
```

### Use Release Build
Always use `--release` flag in production:
```bash
cargo build --release
./target/release/http-server
```

## Common Issues & Solutions

### Port Already in Use
```bash
# Check what's using the port
lsof -i :4221

# Use a different port
cargo run --release -- --port 8080
```

### File Permissions
```bash
# Ensure directory exists and is writable
mkdir -p ./test_files
chmod 755 ./test_files
```

### Dependencies Not Found
```bash
# Update dependencies
cargo update

# Clean and rebuild
cargo clean
cargo build --release
```

## Production Deployment

### Environment Variables
Create a `.env` file:
```bash
HTTP_PORT=80
HTTP_HOST=0.0.0.0
FILE_DIRECTORY=/var/www/files
WORKER_THREADS=16
RUST_LOG=info
```

### Run as Service (systemd)
Create `/etc/systemd/system/http-server.service`:
```ini
[Unit]
Description=Rust HTTP Server
After=network.target

[Service]
Type=simple
User=www-data
WorkingDirectory=/opt/http-server
ExecStart=/opt/http-server/target/release/http-server --port 80 --host 0.0.0.0
Restart=on-failure
Environment="RUST_LOG=info"

[Install]
WantedBy=multi-user.target
```

Enable and start:
```bash
sudo systemctl enable http-server
sudo systemctl start http-server
sudo systemctl status http-server
```

## Monitoring

### View Logs
```bash
# With verbose mode
cargo run --release -- --verbose

# Custom log level
RUST_LOG=debug cargo run --release

# Filter logs
RUST_LOG=codecrafters_http_server=trace cargo run --release
```

### Health Monitoring
```bash
# Simple health check
watch -n 1 'curl -s http://localhost:4221/health | jq'

# With response time
curl -w "@curl-format.txt" -o /dev/null -s http://localhost:4221/health
```

## Development

### Run Tests
```bash
# Unit tests
cargo test

# With output
cargo test -- --nocapture

# Specific test
cargo test test_gzip_compression
```

### Format Code
```bash
cargo fmt
```

### Lint Code
```bash
cargo clippy -- -D warnings
```

### Watch Mode (requires cargo-watch)
```bash
cargo install cargo-watch
cargo watch -x run
```

## Benchmarking

### Using Apache Bench
```bash
# Install
brew install apache-bench  # macOS
apt-get install apache2-utils  # Linux

# Run benchmark
ab -n 10000 -c 100 http://localhost:4221/health

# With keep-alive
ab -n 10000 -c 100 -k http://localhost:4221/health
```

### Using wrk
```bash
# Install
brew install wrk  # macOS

# Run benchmark
wrk -t12 -c400 -d30s http://localhost:4221/health
```

## API Examples

### Using curl
```bash
# POST JSON (if endpoint supported)
curl -X POST \
  -H "Content-Type: application/json" \
  -d '{"key":"value"}' \
  http://localhost:4221/api/endpoint

# Upload binary file
curl -X POST \
  --data-binary @image.png \
  http://localhost:4221/files/image.png

# Custom headers
curl -H "User-Agent: MyApp/1.0" \
  -H "Accept: application/json" \
  http://localhost:4221/api/info
```

### Using Python
```python
import requests

# GET request
response = requests.get('http://localhost:4221/health')
print(response.json())

# POST file
with open('file.txt', 'rb') as f:
    response = requests.post('http://localhost:4221/files/file.txt', data=f)
    print(response.json())

# With compression
response = requests.get(
    'http://localhost:4221/echo/test',
    headers={'Accept-Encoding': 'gzip'}
)
```

### Using JavaScript/Node.js
```javascript
const axios = require('axios');

// GET request
axios.get('http://localhost:4221/health')
  .then(response => console.log(response.data));

// POST file
const FormData = require('form-data');
const fs = require('fs');

const form = new FormData();
form.append('file', fs.createReadStream('file.txt'));

axios.post('http://localhost:4221/files/file.txt', form)
  .then(response => console.log(response.data));
```

## Resources

- **GitHub Repository**: https://github.com/tiwaryash/rust-http.git
- **Full Documentation**: See README.md
- **Testing Guide**: See TESTING.md
- **Rust Documentation**: https://doc.rust-lang.org/

---

**Built with ❤️ in Rust** | [Report Issues](https://github.com/tiwaryash/rust-http/issues)
