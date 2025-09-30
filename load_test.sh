#!/bin/bash

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

echo -e "${BLUE}╔════════════════════════════════════════════════════════════╗${NC}"
echo -e "${BLUE}║     Rust HTTP Server - Load Testing (100 Concurrent)      ║${NC}"
echo -e "${BLUE}╚════════════════════════════════════════════════════════════╝${NC}"
echo ""

# Check if server is running
echo -e "${YELLOW}Checking if server is running...${NC}"
if ! curl -s http://localhost:4221/health > /dev/null 2>&1; then
    echo -e "${RED}Error: Server is not running on port 4221${NC}"
    echo "Please start the server with: cargo run --release -- --verbose"
    exit 1
fi
echo -e "${GREEN}Server is running${NC}"
echo ""

# Test counters
TOTAL_REQUESTS=100
SUCCESS=0
FAILED=0

# Test 1: 100 Concurrent Echo Requests
echo -e "${CYAN}=== Test 1: 100 Concurrent Echo Requests ===${NC}"
echo -e "Sending 100 concurrent requests to /echo endpoint..."

START_TIME=$(date +%s.%N)

for i in {1..100}; do
    (
        response=$(curl -s http://localhost:4221/echo/request-$i 2>&1)
        if [[ $response == "request-$i" ]]; then
            echo "SUCCESS" > /tmp/load_test_$i.result
        else
            echo "FAILED" > /tmp/load_test_$i.result
        fi
    ) &
done

# Wait for all requests to complete
wait

END_TIME=$(date +%s.%N)
DURATION=$(echo "$END_TIME - $START_TIME" | bc)

# Count results
for i in {1..100}; do
    if [ -f /tmp/load_test_$i.result ]; then
        result=$(cat /tmp/load_test_$i.result)
        if [ "$result" == "SUCCESS" ]; then
            ((SUCCESS++))
        else
            ((FAILED++))
        fi
        rm /tmp/load_test_$i.result
    else
        ((FAILED++))
    fi
done

echo -e "${GREEN}Completed: $SUCCESS/$TOTAL_REQUESTS requests succeeded${NC}"
echo -e "${BLUE}Duration: ${DURATION}s${NC}"
echo -e "${BLUE}Throughput: $(echo "scale=2; 100 / $DURATION" | bc) requests/second${NC}"
echo ""

# Test 2: 100 Concurrent Health Checks
echo -e "${CYAN}=== Test 2: 100 Concurrent Health Checks ===${NC}"
echo -e "Sending 100 concurrent requests to /health endpoint..."

SUCCESS2=0
FAILED2=0

START_TIME=$(date +%s.%N)

for i in {1..100}; do
    (
        response=$(curl -s http://localhost:4221/health 2>&1)
        if [[ $response == *"healthy"* ]]; then
            echo "SUCCESS" > /tmp/health_test_$i.result
        else
            echo "FAILED" > /tmp/health_test_$i.result
        fi
    ) &
done

wait

END_TIME=$(date +%s.%N)
DURATION=$(echo "$END_TIME - $START_TIME" | bc)

for i in {1..100}; do
    if [ -f /tmp/health_test_$i.result ]; then
        result=$(cat /tmp/health_test_$i.result)
        if [ "$result" == "SUCCESS" ]; then
            ((SUCCESS2++))
        else
            ((FAILED2++))
        fi
        rm /tmp/health_test_$i.result
    else
        ((FAILED2++))
    fi
done

echo -e "${GREEN}Completed: $SUCCESS2/$TOTAL_REQUESTS requests succeeded${NC}"
echo -e "${BLUE}Duration: ${DURATION}s${NC}"
echo -e "${BLUE}Throughput: $(echo "scale=2; 100 / $DURATION" | bc) requests/second${NC}"
echo ""

# Test 3: 100 Concurrent File Operations
echo -e "${CYAN}=== Test 3: 100 Concurrent File Uploads ===${NC}"
echo -e "Uploading 100 files concurrently..."

SUCCESS3=0
FAILED3=0

mkdir -p test_files

START_TIME=$(date +%s.%N)

for i in {1..100}; do
    (
        response=$(curl -s -X POST -d "Load test file content $i" http://localhost:4221/files/load_test_$i.txt 2>&1)
        if [[ $response == *"uploaded successfully"* ]] || [[ $response == *"File uploaded"* ]]; then
            echo "SUCCESS" > /tmp/upload_test_$i.result
        else
            echo "FAILED" > /tmp/upload_test_$i.result
        fi
    ) &
done

wait

END_TIME=$(date +%s.%N)
DURATION=$(echo "$END_TIME - $START_TIME" | bc)

for i in {1..100}; do
    if [ -f /tmp/upload_test_$i.result ]; then
        result=$(cat /tmp/upload_test_$i.result)
        if [ "$result" == "SUCCESS" ]; then
            ((SUCCESS3++))
        else
            ((FAILED3++))
        fi
        rm /tmp/upload_test_$i.result
    else
        ((FAILED3++))
    fi
done

echo -e "${GREEN}Completed: $SUCCESS3/$TOTAL_REQUESTS uploads succeeded${NC}"
echo -e "${BLUE}Duration: ${DURATION}s${NC}"
echo -e "${BLUE}Throughput: $(echo "scale=2; 100 / $DURATION" | bc) requests/second${NC}"
echo ""

# Test 4: 100 Concurrent File Downloads
echo -e "${CYAN}=== Test 4: 100 Concurrent File Downloads ===${NC}"
echo -e "Downloading 100 files concurrently..."

SUCCESS4=0
FAILED4=0

START_TIME=$(date +%s.%N)

for i in {1..100}; do
    (
        response=$(curl -s http://localhost:4221/files/load_test_$i.txt 2>&1)
        if [[ $response == *"Load test file content $i"* ]]; then
            echo "SUCCESS" > /tmp/download_test_$i.result
        else
            echo "FAILED" > /tmp/download_test_$i.result
        fi
    ) &
done

wait

END_TIME=$(date +%s.%N)
DURATION=$(echo "$END_TIME - $START_TIME" | bc)

for i in {1..100}; do
    if [ -f /tmp/download_test_$i.result ]; then
        result=$(cat /tmp/download_test_$i.result)
        if [ "$result" == "SUCCESS" ]; then
            ((SUCCESS4++))
        else
            ((FAILED4++))
        fi
        rm /tmp/download_test_$i.result
    else
        ((FAILED4++))
    fi
done

echo -e "${GREEN}Completed: $SUCCESS4/$TOTAL_REQUESTS downloads succeeded${NC}"
echo -e "${BLUE}Duration: ${DURATION}s${NC}"
echo -e "${BLUE}Throughput: $(echo "scale=2; 100 / $DURATION" | bc) requests/second${NC}"
echo ""

# Test 5: 100 Concurrent Compression Requests
echo -e "${CYAN}=== Test 5: 100 Concurrent Compressed Requests ===${NC}"
echo -e "Sending 100 concurrent requests with compression..."

SUCCESS5=0
FAILED5=0

START_TIME=$(date +%s.%N)

for i in {1..100}; do
    (
        headers=$(curl -s -i -H "Accept-Encoding: gzip" http://localhost:4221/echo/compressed-request-$i 2>&1 | head -20)
        if [[ $headers == *"Content-Encoding: gzip"* ]]; then
            echo "SUCCESS" > /tmp/compress_test_$i.result
        else
            echo "FAILED" > /tmp/compress_test_$i.result
        fi
    ) &
done

wait

END_TIME=$(date +%s.%N)
DURATION=$(echo "$END_TIME - $START_TIME" | bc)

for i in {1..100}; do
    if [ -f /tmp/compress_test_$i.result ]; then
        result=$(cat /tmp/compress_test_$i.result)
        if [ "$result" == "SUCCESS" ]; then
            ((SUCCESS5++))
        else
            ((FAILED5++))
        fi
        rm /tmp/compress_test_$i.result
    else
        ((FAILED5++))
    fi
done

echo -e "${GREEN}Completed: $SUCCESS5/$TOTAL_REQUESTS compressed requests succeeded${NC}"
echo -e "${BLUE}Duration: ${DURATION}s${NC}"
echo -e "${BLUE}Throughput: $(echo "scale=2; 100 / $DURATION" | bc) requests/second${NC}"
echo ""

# Cleanup uploaded test files
echo -e "${YELLOW}Cleaning up test files...${NC}"
for i in {1..100}; do
    curl -s -X DELETE http://localhost:4221/files/load_test_$i.txt > /dev/null 2>&1
done
echo -e "${GREEN}Cleanup complete${NC}"
echo ""

# Summary
TOTAL_TESTS=500
TOTAL_SUCCESS=$((SUCCESS + SUCCESS2 + SUCCESS3 + SUCCESS4 + SUCCESS5))
TOTAL_FAILED=$((FAILED + FAILED2 + FAILED3 + FAILED4 + FAILED5))

echo -e "${BLUE}╔════════════════════════════════════════════════════════════╗${NC}"
echo -e "${BLUE}║                     Load Test Summary                      ║${NC}"
echo -e "${BLUE}╠════════════════════════════════════════════════════════════╣${NC}"
echo -e "${BLUE}║${NC}  ${CYAN}Test 1 - Echo Requests:${NC}      $SUCCESS/100"
echo -e "${BLUE}║${NC}  ${CYAN}Test 2 - Health Checks:${NC}      $SUCCESS2/100"
echo -e "${BLUE}║${NC}  ${CYAN}Test 3 - File Uploads:${NC}       $SUCCESS3/100"
echo -e "${BLUE}║${NC}  ${CYAN}Test 4 - File Downloads:${NC}     $SUCCESS4/100"
echo -e "${BLUE}║${NC}  ${CYAN}Test 5 - Compressed Requests:${NC} $SUCCESS5/100"
echo -e "${BLUE}╠════════════════════════════════════════════════════════════╣${NC}"
echo -e "${BLUE}║${NC}  ${GREEN}Total Successful: $TOTAL_SUCCESS/$TOTAL_TESTS${NC}"
echo -e "${BLUE}║${NC}  ${RED}Total Failed: $TOTAL_FAILED/$TOTAL_TESTS${NC}"
echo -e "${BLUE}╚════════════════════════════════════════════════════════════╝${NC}"
echo ""

if [ $TOTAL_FAILED -eq 0 ]; then
    echo -e "${GREEN}SUCCESS! All 500 concurrent requests handled perfectly!${NC}"
    echo -e "${GREEN}Your server is production-ready for high concurrency!${NC}"
    exit 0
else
    SUCCESS_RATE=$(echo "scale=2; ($TOTAL_SUCCESS * 100) / $TOTAL_TESTS" | bc)
    echo -e "${YELLOW}Load test completed with $SUCCESS_RATE% success rate${NC}"
    exit 1
fi
