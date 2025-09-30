# Testing Guide for Rust HTTP Server

This guide provides comprehensive testing instructions for all features of the HTTP server.

## Prerequisites

Start the server in one terminal:

```bash
cargo run --release -- --verbose --directory ./test_files
```

## 1. Basic Connection Tests

### Test: Bind to a port
```bash
# Server should start without errors on port 4221
netstat -an | grep 4221
```

Expected: Server is listening on 127.0.0.1:4221

## 2. Response Code Tests

### Test: Respond with 200 OK
```bash
curl -i http://localhost:4221/
```

Expected: `HTTP/1.1 200 OK` status code with HTML response

### Test: 404 Not Found
```bash
curl -i http://localhost:4221/nonexistent
```

Expected: `HTTP/1.1 404 Not Found` status code

## 3. URL Path Extraction

### Test: Echo endpoint
```bash
curl http://localhost:4221/echo/HelloWorld
```

Expected: `HelloWorld`

```bash
curl http://localhost:4221/echo/This%20is%20a%20test
```

Expected: `This%20is%20a%20test`

## 4. Response Body Tests

### Test: Text response
```bash
curl http://localhost:4221/user-agent
```

Expected: Your user agent string

### Test: JSON response
```bash
curl http://localhost:4221/api/info
```

Expected: JSON with server information

### Test: HTML response
```bash
curl http://localhost:4221/
```

Expected: HTML welcome page

## 5. Header Reading Tests

### Test: Read User-Agent header
```bash
curl -H "User-Agent: MyCustomAgent/1.0" http://localhost:4221/user-agent
```

Expected: `MyCustomAgent/1.0`

### Test: View all headers
```bash
curl -H "Custom-Header: TestValue" http://localhost:4221/headers
```

Expected: JSON object with all request headers including custom header

## 6. Concurrent Connection Tests

### Test: Multiple simultaneous requests
```bash
# Run in parallel
for i in {1..10}; do
  curl http://localhost:4221/echo/request-$i &
done
wait
```

Expected: All 10 requests should complete successfully

### Test: Load test with Apache Bench (if installed)
```bash
ab -n 1000 -c 10 http://localhost:4221/health
```

Expected: All requests successful with good throughput

## 7. File Operations Tests

### Test: Upload a file (POST)
```bash
# Create test content
echo "Hello, World! This is test content." > /tmp/test.txt

# Upload file
curl -X POST --data-binary @/tmp/test.txt http://localhost:4221/files/test.txt
```

Expected: `HTTP/1.1 201 Created` with JSON confirmation

### Test: Download a file (GET)
```bash
curl http://localhost:4221/files/test.txt
```

Expected: File contents: `Hello, World! This is test content.`

### Test: Delete a file (DELETE)
```bash
curl -X DELETE http://localhost:4221/files/test.txt
```

Expected: JSON confirmation of deletion

### Test: Get non-existent file
```bash
curl -i http://localhost:4221/files/nonexistent.txt
```

Expected: `HTTP/1.1 404 Not Found`

### Test: Content-Type detection
```bash
# Upload HTML file
echo "<h1>Test</h1>" > /tmp/test.html
curl -X POST --data-binary @/tmp/test.html http://localhost:4221/files/test.html

# Download and check Content-Type
curl -i http://localhost:4221/files/test.html | grep Content-Type
```

Expected: `Content-Type: text/html`

## 8. Request Body Tests

### Test: POST with body
```bash
curl -X POST \
  -H "Content-Type: text/plain" \
  -d "This is the request body" \
  http://localhost:4221/files/body-test.txt
```

Expected: File created with the body content

### Test: Large body
```bash
# Create 1MB file
dd if=/dev/urandom of=/tmp/large.bin bs=1024 count=1024

# Upload large file
curl -X POST --data-binary @/tmp/large.bin http://localhost:4221/files/large.bin

# Verify size
curl -i http://localhost:4221/files/large.bin | grep Content-Length
```

Expected: Content-Length: 1048576

## 9. HTTP Compression Tests

### Test: Gzip compression
```bash
curl -H "Accept-Encoding: gzip" \
     --compressed \
     http://localhost:4221/echo/This-is-a-test-string-that-should-be-compressed
```

Expected: Compressed response (automatic with --compressed flag)

### Test: Compression headers
```bash
curl -i -H "Accept-Encoding: gzip" http://localhost:4221/echo/test | grep Content-Encoding
```

Expected: `Content-Encoding: gzip`

### Test: Multiple compression schemes
```bash
# Request with multiple encodings (server picks best)
curl -i -H "Accept-Encoding: br, gzip, deflate" \
     http://localhost:4221/echo/compression-test | grep Content-Encoding
```

Expected: `Content-Encoding: br` (Brotli, the best compression)

### Test: Deflate compression
```bash
curl -i -H "Accept-Encoding: deflate" \
     http://localhost:4221/echo/deflate-test | grep Content-Encoding
```

Expected: `Content-Encoding: deflate`

### Test: Brotli compression
```bash
curl -i -H "Accept-Encoding: br" \
     http://localhost:4221/echo/brotli-test | grep Content-Encoding
```

Expected: `Content-Encoding: br`

### Test: No compression
```bash
curl -i http://localhost:4221/echo/no-compression | grep Content-Encoding
```

Expected: No Content-Encoding header (uncompressed)

### Test: Compression effectiveness
```bash
# Create large text file
python3 -c "print('Hello World! ' * 1000)" > /tmp/large.txt

# Upload
curl -X POST --data-binary @/tmp/large.txt http://localhost:4221/files/large.txt

# Compare sizes
echo "Uncompressed:"
curl -i http://localhost:4221/files/large.txt | grep Content-Length

echo "Compressed:"
curl -i -H "Accept-Encoding: gzip" http://localhost:4221/files/large.txt | grep Content-Length
```

Expected: Compressed size significantly smaller

## 10. Health & Monitoring Tests

### Test: Health check
```bash
curl http://localhost:4221/health
```

Expected: JSON with status "healthy" and timestamp

### Test: Server info
```bash
curl http://localhost:4221/api/info | jq
```

Expected: Formatted JSON with server details and features

## 11. Error Handling Tests

### Test: Invalid method
```bash
curl -X INVALID http://localhost:4221/
```

Expected: Error response

### Test: Malformed request
```bash
echo -e "INVALID REQUEST\r\n\r\n" | nc localhost 4221
```

Expected: Error response or connection close

### Test: Path traversal attempt
```bash
curl http://localhost:4221/files/../../../etc/passwd
```

Expected: 400 Bad Request (security protection)

## 12. Configuration Tests

### Test: Custom port
```bash
# Start server on different port
cargo run --release -- --port 8080 &
SERVER_PID=$!

# Test
curl http://localhost:8080/health

# Cleanup
kill $SERVER_PID
```

### Test: Custom directory
```bash
mkdir -p /tmp/custom-files
echo "Custom directory test" > /tmp/custom-files/test.txt

# Start server with custom directory
cargo run --release -- --directory /tmp/custom-files &
SERVER_PID=$!

# Test
curl http://localhost:4221/files/test.txt

# Cleanup
kill $SERVER_PID
rm -rf /tmp/custom-files
```

### Test: Custom worker threads
```bash
cargo run --release -- --workers 8 --verbose
```

Expected: Log shows "Worker threads: 8"

## 13. Performance Tests

### Test: Response time
```bash
time curl http://localhost:4221/health
```

Expected: Response in < 50ms

### Test: Throughput
```bash
# Using Apache Bench (install with: brew install apache-bench or apt-get install apache2-utils)
ab -n 10000 -c 100 http://localhost:4221/health
```

Expected: > 1000 requests/second

## 14. Integration Tests

### Test: Full workflow
```bash
#!/bin/bash

# 1. Check server health
echo "1. Checking health..."
curl -s http://localhost:4221/health | jq .status

# 2. Upload a file
echo "2. Uploading file..."
echo "Integration test content" > /tmp/integration.txt
curl -s -X POST --data-binary @/tmp/integration.txt \
  http://localhost:4221/files/integration.txt | jq .message

# 3. Download the file
echo "3. Downloading file..."
curl -s http://localhost:4221/files/integration.txt

# 4. Test compression
echo "4. Testing compression..."
curl -s -i -H "Accept-Encoding: gzip" \
  http://localhost:4221/files/integration.txt | grep Content-Encoding

# 5. Delete the file
echo "5. Deleting file..."
curl -s -X DELETE http://localhost:4221/files/integration.txt | jq .message

# 6. Verify deletion
echo "6. Verifying deletion..."
curl -s -i http://localhost:4221/files/integration.txt | head -1

echo "Integration test complete!"
```

## Automated Test Script

Create a file `run_tests.sh`:

```bash
#!/bin/bash

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
NC='\033[0m' # No Color

# Test counter
PASSED=0
FAILED=0

test_endpoint() {
    local name=$1
    local url=$2
    local expected=$3
    
    echo -n "Testing: $name... "
    response=$(curl -s "$url")
    
    if [[ $response == *"$expected"* ]]; then
        echo -e "${GREEN}PASSED${NC}"
        ((PASSED++))
    else
        echo -e "${RED}FAILED${NC}"
        ((FAILED++))
    fi
}

echo "Starting HTTP Server Tests"
echo "=========================="

# Basic tests
test_endpoint "Root endpoint" "http://localhost:4221/" "Rust HTTP Server"
test_endpoint "Health check" "http://localhost:4221/health" "healthy"
test_endpoint "Echo service" "http://localhost:4221/echo/test123" "test123"
test_endpoint "User agent" "http://localhost:4221/user-agent" "curl"
test_endpoint "API info" "http://localhost:4221/api/info" "version"

# File operations
curl -s -X POST -d "test" http://localhost:4221/files/test.txt > /dev/null
test_endpoint "File upload" "http://localhost:4221/files/test.txt" "test"
curl -s -X DELETE http://localhost:4221/files/test.txt > /dev/null

# Compression
response=$(curl -s -i -H "Accept-Encoding: gzip" http://localhost:4221/echo/compression-test | grep -i "Content-Encoding")
if [[ $response == *"gzip"* ]]; then
    echo -e "Testing: Compression... ${GREEN}PASSED${NC}"
    ((PASSED++))
else
    echo -e "Testing: Compression... ${RED}FAILED${NC}"
    ((FAILED++))
fi

echo "=========================="
echo "Tests passed: $PASSED"
echo "Tests failed: $FAILED"

if [ $FAILED -eq 0 ]; then
    echo -e "${GREEN}All tests passed!${NC}"
    exit 0
else
    echo -e "${RED}Some tests failed!${NC}"
    exit 1
fi
```

Make it executable and run:
```bash
chmod +x run_tests.sh
./run_tests.sh
```

## Cleanup

After testing, clean up test files:

```bash
rm -rf test_files/
rm -f /tmp/test.txt /tmp/test.html /tmp/large.bin /tmp/large.txt
```

---

## Troubleshooting

### Server won't start
- Check if port 4221 is already in use: `lsof -i :4221`
- Try a different port: `cargo run -- --port 8080`

### Connection refused
- Ensure server is running: `ps aux | grep http-server`
- Check firewall settings

### File operations fail
- Check directory permissions
- Verify the directory exists or will be created
- Check disk space

### Compression not working
- Verify Accept-Encoding header is set
- Check response size (compression only for > 100 bytes)
- Use `--compressed` flag with curl for automatic decompression
