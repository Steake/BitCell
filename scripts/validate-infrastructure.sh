#!/bin/bash
# BitCell Production Infrastructure Validation Script

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
COMPOSE_FILE="$SCRIPT_DIR/../infra/docker/docker-compose.yml"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo "========================================"
echo "BitCell Infrastructure Validation"
echo "========================================"

# Check prerequisites
echo -e "\n${YELLOW}Checking prerequisites...${NC}"

if ! command -v docker &> /dev/null; then
    echo -e "${RED}✗ Docker not found${NC}"
    exit 1
fi
echo -e "${GREEN}✓ Docker found${NC}"

if ! command -v docker-compose &> /dev/null; then
    echo -e "${RED}✗ Docker Compose not found${NC}"
    exit 1
fi
echo -e "${GREEN}✓ Docker Compose found${NC}"

# Check if infrastructure is running
echo -e "\n${YELLOW}Checking infrastructure status...${NC}"

if ! docker-compose -f "$COMPOSE_FILE" ps | grep -q "Up"; then
    echo -e "${YELLOW}! Infrastructure not running. Starting...${NC}"
    
    # Check if image exists, build if not
    if ! docker images | grep -q "bitcell-node.*latest"; then
        echo "Building BitCell node image..."
        docker build -f "$SCRIPT_DIR/../infra/docker/Dockerfile" -t bitcell-node:latest "$SCRIPT_DIR/.."
    else
        echo "BitCell node image already exists, skipping build..."
    fi
    
    # Start infrastructure
    echo "Starting infrastructure..."
    docker-compose -f "$COMPOSE_FILE" up -d
    
    echo "Waiting 30 seconds for services to start..."
    sleep 30
else
    echo -e "${GREEN}✓ Infrastructure is running${NC}"
fi

# Test health endpoints
echo -e "\n${YELLOW}Testing health endpoints...${NC}"

HEALTH_PASS=0
HEALTH_FAIL=0

for port in 9090 9091 9092 9093 9094 9095 9096; do
    if curl -sf "http://localhost:$port/health" > /dev/null 2>&1; then
        echo -e "${GREEN}✓ Health check passed for port $port${NC}"
        ((HEALTH_PASS++))
    else
        echo -e "${RED}✗ Health check failed for port $port${NC}"
        ((HEALTH_FAIL++))
    fi
done

echo "Health checks: $HEALTH_PASS passed, $HEALTH_FAIL failed"

# Test metrics endpoints
echo -e "\n${YELLOW}Testing metrics endpoints...${NC}"

METRICS_PASS=0
METRICS_FAIL=0

for port in 9090 9091 9092; do
    if curl -sf "http://localhost:$port/metrics" | grep -q "bitcell_chain_height"; then
        echo -e "${GREEN}✓ Metrics available on port $port${NC}"
        ((METRICS_PASS++))
    else
        echo -e "${RED}✗ Metrics not available on port $port${NC}"
        ((METRICS_FAIL++))
    fi
done

echo "Metrics checks: $METRICS_PASS passed, $METRICS_FAIL failed"

# Test Prometheus
echo -e "\n${YELLOW}Testing Prometheus...${NC}"

if curl -sf "http://localhost:9999/api/v1/targets" > /dev/null 2>&1; then
    echo -e "${GREEN}✓ Prometheus is accessible${NC}"
    
    # Check targets
    TARGETS=$(curl -s "http://localhost:9999/api/v1/targets" | grep -o '"health":"up"' | wc -l)
    echo "  Active targets: $TARGETS"
else
    echo -e "${RED}✗ Prometheus is not accessible${NC}"
fi

# Test Grafana
echo -e "\n${YELLOW}Testing Grafana...${NC}"

if curl -sf "http://localhost:3000/api/health" > /dev/null 2>&1; then
    echo -e "${GREEN}✓ Grafana is accessible${NC}"
else
    echo -e "${RED}✗ Grafana is not accessible${NC}"
fi

# Test Alertmanager
echo -e "\n${YELLOW}Testing Alertmanager...${NC}"

if curl -sf "http://localhost:9093/api/v1/status" > /dev/null 2>&1; then
    echo -e "${GREEN}✓ Alertmanager is accessible${NC}"
else
    echo -e "${RED}✗ Alertmanager is not accessible${NC}"
fi

# Test HAProxy
echo -e "\n${YELLOW}Testing HAProxy...${NC}"

if curl -sf "http://localhost:8404" > /dev/null 2>&1; then
    echo -e "${GREEN}✓ HAProxy stats page is accessible${NC}"
else
    echo -e "${RED}✗ HAProxy stats page is not accessible${NC}"
fi

# Test load balancer
echo -e "\n${YELLOW}Testing load balancer...${NC}"

if curl -sf "http://localhost:80/health" > /dev/null 2>&1; then
    echo -e "${GREEN}✓ Load balancer is routing requests${NC}"
else
    echo -e "${YELLOW}! Load balancer health check failed (nodes may not be ready)${NC}"
fi

# Check Docker network
echo -e "\n${YELLOW}Checking Docker network...${NC}"

if docker network inspect bitcell_bitcell-net > /dev/null 2>&1; then
    echo -e "${GREEN}✓ BitCell network exists${NC}"
    CONTAINERS=$(docker network inspect bitcell_bitcell-net | grep -o '"Name": "bitcell-' | wc -l)
    echo "  Connected containers: $CONTAINERS"
else
    echo -e "${RED}✗ BitCell network not found${NC}"
fi

# Summary
echo -e "\n========================================"
echo "Validation Summary"
echo "========================================"

TOTAL_PASS=$((HEALTH_PASS + METRICS_PASS))
TOTAL_FAIL=$((HEALTH_FAIL + METRICS_FAIL))

echo -e "Health checks: ${GREEN}$HEALTH_PASS passed${NC}, ${RED}$HEALTH_FAIL failed${NC}"
echo -e "Metrics checks: ${GREEN}$METRICS_PASS passed${NC}, ${RED}$METRICS_FAIL failed${NC}"
echo -e "\nTotal: ${GREEN}$TOTAL_PASS passed${NC}, ${RED}$TOTAL_FAIL failed${NC}"

if [ $TOTAL_FAIL -eq 0 ]; then
    echo -e "\n${GREEN}✓ All validation checks passed!${NC}"
    exit 0
elif [ $TOTAL_PASS -gt 0 ]; then
    echo -e "\n${YELLOW}! Some validation checks failed${NC}"
    echo "Check the output above for details"
    exit 1
else
    echo -e "\n${RED}✗ All validation checks failed${NC}"
    echo "Infrastructure may not be running properly"
    exit 1
fi
