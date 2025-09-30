#!/bin/bash

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Test counters
PASSED=0
FAILED=0

echo -e "${BLUE}╔═══════════════════════════════════════════════════╗${NC}"
echo -e "${BLUE}║     Rust HTTP Server - Comprehensive Tests       ║${NC}"
echo -e "${BLUE}╔═══════════════════════════════════════════════════╗${NC}"
echo ""

# Function to test an endpoint
test_endpoint() {
    local name="$1"
    local url="$2"
    local expected="$3"
    local method="${4:-GET}"
    local data="$5"
    
    echo -n "Testing: $name... "
    
    if [ -n "$data" ]; then
        response=$(curl -s -X "$method" -d "$data" "$url")
    else
        response=$(curl -s -X "$method" "$url")
    fi
    
    if [[ $response == *"$expected"* ]]; then
        echo -e "${GREEN}PASSED${NC}"
        ((PASSED++))
    else
        echo -e "${RED}FAILED${NC}"
        echo -e "  Expected: $expected"
        echo -e "  Got: $response"
        ((FAILED++))
    fi
}

# Check if server is running
echo -e "${YELLOW}Checking if server is running...${NC}"
if ! curl -s http://localhost:4221/health > /dev/null 2>&1; then
    echo -e "${RED}Error: Server is not running on port 4221${NC}"
    echo "Please start the server with: cargo run --release -- --verbose"
    exit 1
fi
echo -e "${GREEN}Server is running${NC}"
echo ""

# Basic endpoint tests
echo -e "${BLUE}=== Basic Endpoint Tests ===${NC}"
test_endpoint "Root endpoint" "http://localhost:4221/" "Rust HTTP Server"
test_endpoint "Health check" "http://localhost:4221/health" "healthy"
test_endpoint "Echo service" "http://localhost:4221/echo/HelloWorld" "HelloWorld"
test_endpoint "User agent" "http://localhost:4221/user-agent" "curl"
test_endpoint "API info" "http://localhost:4221/api/info" "version"
test_endpoint "Headers endpoint" "http://localhost:4221/headers" "user-agent"
echo ""

# HTTP status code tests
echo -e "${BLUE}=== HTTP Status Code Tests ===${NC}"
status_code=$(curl -s -o /dev/null -w "%{http_code}" http://localhost:4221/)
if [ "$status_code" == "200" ]; then
    echo -e "Testing: 200 OK status... ${GREEN}PASSED${NC}"
    ((PASSED++))
else
    echo -e "Testing: 200 OK status... ${RED}FAILED${NC}"
    ((FAILED++))
fi

status_code=$(curl -s -o /dev/null -w "%{http_code}" http://localhost:4221/nonexistent)
if [ "$status_code" == "404" ]; then
    echo -e "Testing: 404 Not Found status... ${GREEN}PASSED${NC}"
    ((PASSED++))
else
    echo -e "Testing: 404 Not Found status... ${RED}FAILED${NC}"
    ((FAILED++))
fi
echo ""

# File operations tests
echo -e "${BLUE}=== File Operations Tests ===${NC}"

# Create test directory
mkdir -p test_files

# Upload a file
echo "Test file content" > /tmp/test_upload.txt
test_endpoint "File upload (POST)" "http://localhost:4221/files/test_upload.txt" "uploaded successfully" "POST" "$(cat /tmp/test_upload.txt)"

# Download the file
test_endpoint "File download (GET)" "http://localhost:4221/files/test_upload.txt" "Test file content"

# Delete the file
test_endpoint "File deletion (DELETE)" "http://localhost:4221/files/test_upload.txt" "deleted successfully" "DELETE"

# Verify deletion
status_code=$(curl -s -o /dev/null -w "%{http_code}" http://localhost:4221/files/test_upload.txt)
if [ "$status_code" == "404" ]; then
    echo -e "Testing: File deleted verification... ${GREEN}PASSED${NC}"
    ((PASSED++))
else
    echo -e "Testing: File deleted verification... ${RED}FAILED${NC}"
    ((FAILED++))
fi
echo ""

# Compression tests
echo -e "${BLUE}=== Compression Tests ===${NC}"

# Test Gzip compression
response=$(curl -s -i -H "Accept-Encoding: gzip" http://localhost:4221/echo/compression-test-string | grep -a -i "Content-Encoding")
if [[ $response == *"gzip"* ]]; then
    echo -e "Testing: Gzip compression... ${GREEN}PASSED${NC}"
    ((PASSED++))
else
    echo -e "Testing: Gzip compression... ${RED}FAILED${NC}"
    ((FAILED++))
fi

# Test Deflate compression
response=$(curl -s -i -H "Accept-Encoding: deflate" http://localhost:4221/echo/deflate-test-string | grep -a -i "Content-Encoding")
if [[ $response == *"deflate"* ]]; then
    echo -e "Testing: Deflate compression... ${GREEN}PASSED${NC}"
    ((PASSED++))
else
    echo -e "Testing: Deflate compression... ${RED}FAILED${NC}"
    ((FAILED++))
fi

# Test Brotli compression (best)
response=$(curl -s -i -H "Accept-Encoding: br, gzip, deflate" http://localhost:4221/echo/brotli-test-string | grep -a -i "Content-Encoding")
if [[ $response == *"br"* ]]; then
    echo -e "Testing: Brotli compression (priority)... ${GREEN}PASSED${NC}"
    ((PASSED++))
else
    echo -e "Testing: Brotli compression (priority)... ${RED}FAILED${NC}"
    ((FAILED++))
fi
echo ""

# Concurrent connections test
echo -e "${BLUE}=== Concurrent Connections Test ===${NC}"
echo -n "Testing: 10 concurrent requests... "

success_count=0
for i in {1..10}; do
    response=$(curl -s http://localhost:4221/echo/request-$i) &
done
wait

# Count successful responses
for i in {1..10}; do
    response=$(curl -s http://localhost:4221/health)
    if [[ $response == *"healthy"* ]]; then
        ((success_count++))
    fi
done

if [ $success_count -eq 10 ]; then
    echo -e "${GREEN}PASSED${NC}"
    ((PASSED++))
else
    echo -e "${RED}FAILED${NC} ($success_count/10 succeeded)"
    ((FAILED++))
fi
echo ""

# Security tests
echo -e "${BLUE}=== Security Tests ===${NC}"

# Path traversal attempt
status_code=$(curl -s -o /dev/null -w "%{http_code}" http://localhost:4221/files/../../../etc/passwd)
if [ "$status_code" == "400" ] || [ "$status_code" == "404" ]; then
    echo -e "Testing: Path traversal protection... ${GREEN}PASSED${NC}"
    ((PASSED++))
else
    echo -e "Testing: Path traversal protection... ${RED}FAILED${NC}"
    ((FAILED++))
fi
echo ""

# Summary
echo -e "${BLUE}╔═══════════════════════════════════════════════════╗${NC}"
echo -e "${BLUE}║                   Test Summary                    ║${NC}"
echo -e "${BLUE}╠═══════════════════════════════════════════════════╣${NC}"
echo -e "${BLUE}║${NC}  ${GREEN}Tests Passed: $PASSED${NC}"
echo -e "${BLUE}║${NC}  ${RED}Tests Failed: $FAILED${NC}"
echo -e "${BLUE}║${NC}  Total Tests:  $((PASSED + FAILED))"
echo -e "${BLUE}╚═══════════════════════════════════════════════════╝${NC}"

if [ $FAILED -eq 0 ]; then
    echo -e "\n${GREEN}All tests passed! Your HTTP server is working perfectly!${NC}\n"
    exit 0
else
    echo -e "\n${YELLOW}Some tests failed. Please review the output above.${NC}\n"
    exit 1
fi

