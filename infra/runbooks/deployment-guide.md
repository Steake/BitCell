# BitCell Production Deployment Guide

## Overview

This guide covers deploying BitCell infrastructure for production use across multiple regions.

---

## Prerequisites

### System Requirements

**Per Node:**
- CPU: 4+ cores (8+ recommended)
- RAM: 8GB minimum (16GB recommended)
- Storage: 100GB SSD (fast NVMe preferred)
- Network: 1Gbps+ bandwidth, <200ms cross-region latency

**Operating System:**
- Linux (Ubuntu 22.04 LTS recommended)
- Docker 24.0+
- Docker Compose 2.20+
- OR Kubernetes 1.28+

### Cloud Provider Recommendations

**Multi-Region Setup (4+ regions):**

**AWS:**
- us-east-1 (N. Virginia)
- us-west-1 (N. California)
- eu-central-1 (Frankfurt)
- ap-southeast-1 (Singapore)

**GCP:**
- us-east1 (South Carolina)
- us-west1 (Oregon)
- europe-west1 (Belgium)
- asia-southeast1 (Singapore)

**Azure:**
- East US
- West US 2
- West Europe
- Southeast Asia

---

## Deployment Options

### Option 1: Docker Compose (Recommended for Testing/Small Scale)

#### Quick Start

```bash
# Clone repository
git clone https://github.com/Steake/BitCell.git
cd BitCell

# Build node image
docker build -f infra/docker/Dockerfile -t bitcell-node:latest .

# Set required environment variables
export GRAFANA_ADMIN_PASSWORD='your-secure-password-here'

# Start infrastructure
cd infra/docker
docker-compose up -d

# Verify deployment
docker-compose ps
```

#### Access Services

- **Grafana**: http://localhost:3000 (admin/<your-password>)
- **Prometheus**: http://localhost:9999
- **Alertmanager**: http://localhost:9093
- **HAProxy Stats**: http://localhost:8404
- **Node RPC**: http://localhost:8545-8551

#### Monitoring Health

```bash
# Check all services
docker-compose ps

# View logs
docker-compose logs -f node-us-east-1

# Check metrics
curl http://localhost:9090/metrics

# Run health checks
for port in 9090 9091 9092 9093 9094 9095 9096; do
    echo "Checking port $port..."
    curl -s http://localhost:$port/health | head -1
done
```

---

### Option 2: Kubernetes (Recommended for Production)

#### Prerequisites

- Kubernetes cluster with 3+ regions
- kubectl configured
- Persistent storage provisioner
- Load balancer support

#### Deploy to Kubernetes

```bash
# Create namespace
kubectl create namespace bitcell-production

# Create secrets
kubectl create secret generic grafana-secret \
    --from-literal=admin-password='YOUR_SECURE_PASSWORD' \
    -n bitcell-production

# Deploy infrastructure
kubectl apply -f infra/kubernetes/deployment.yaml

# Verify deployment
kubectl get pods -n bitcell-production
kubectl get svc -n bitcell-production
```

#### Scale Nodes

```bash
# Scale US-East region
kubectl scale statefulset bitcell-node-us-east \
    --replicas=3 -n bitcell-production

# Scale globally
kubectl scale statefulset bitcell-node-us-west --replicas=3 -n bitcell-production
kubectl scale statefulset bitcell-node-eu-central --replicas=3 -n bitcell-production
```

#### Monitoring

```bash
# Get service endpoints
kubectl get svc -n bitcell-production

# Port forward Grafana
kubectl port-forward svc/grafana 3000:3000 -n bitcell-production

# View logs
kubectl logs -f statefulset/bitcell-node-us-east -n bitcell-production
```

---

## Configuration

### Environment Variables

**Required:**
- `REGION`: Geographic region (us-east, us-west, eu-central, ap-southeast)
- `NODE_ID`: Unique node identifier
- `P2P_PORT`: Peer-to-peer communication port (default: 9000)
- `RPC_PORT`: JSON-RPC API port (default: 8545)
- `METRICS_PORT`: Prometheus metrics port (default: 9090)

**Optional:**
- `DATA_DIR`: Data directory path (default: /data/bitcell)
- `LOG_LEVEL`: Logging level (debug, info, warn, error)
- `BOOTSTRAP_NODES`: Comma-separated list of bootstrap peers
- `ENABLE_DHT`: Enable DHT peer discovery (true/false)
- `KEY_SEED`: Deterministic key generation seed

### Network Configuration

**Firewall Rules:**

**Inbound:**
- P2P: 9000-9010 (TCP/UDP)
- RPC: 8545-8555 (TCP)
- Metrics: 9090-9100 (TCP, restricted to monitoring subnet)

**Outbound:**
- Allow all (for peer discovery and cross-region communication)

**Security Groups (AWS example):**

```bash
# Create security group
aws ec2 create-security-group \
    --group-name bitcell-node \
    --description "BitCell node security group"

# Add rules
aws ec2 authorize-security-group-ingress \
    --group-name bitcell-node \
    --protocol tcp --port 9000-9010 --cidr 0.0.0.0/0

aws ec2 authorize-security-group-ingress \
    --group-name bitcell-node \
    --protocol tcp --port 8545 --cidr 0.0.0.0/0
```

---

## Monitoring Setup

### Prometheus Configuration

Prometheus automatically discovers nodes via service discovery:

**Docker Compose:** Static configuration in `infra/monitoring/prometheus.yml`

**Kubernetes:** Uses pod annotations:
```yaml
annotations:
  prometheus.io/scrape: "true"
  prometheus.io/port: "9090"
  prometheus.io/path: "/metrics"
```

### Grafana Dashboards

**Pre-configured dashboards:**
1. Production Overview - High-level metrics
2. Node Details - Per-node performance
3. Network Health - Peer connectivity
4. EBSL Trust System - Miner reputation

**Import additional dashboards:**
1. Login to Grafana
2. Navigate to Dashboards → Import
3. Upload JSON from `infra/monitoring/grafana/dashboards/`

### Alert Configuration

**Configure Alertmanager:**

Edit `infra/monitoring/alertmanager.yml`:

```yaml
global:
  slack_api_url: 'YOUR_SLACK_WEBHOOK'

receivers:
  - name: 'pagerduty-critical'
    pagerduty_configs:
      - service_key: 'YOUR_PAGERDUTY_KEY'
```

**Test alerts:**

```bash
# Manually trigger test alert
curl -X POST http://localhost:9093/api/v1/alerts \
    -H "Content-Type: application/json" \
    -d '[{
        "labels": {
            "alertname": "TestAlert",
            "severity": "warning"
        },
        "annotations": {
            "summary": "This is a test alert"
        }
    }]'
```

---

## High Availability

### Regional Redundancy

**Minimum per region:** 2 nodes
**Recommended:** 3+ nodes per region

### Load Balancing

**HAProxy (included in Docker Compose):**
- Automatic health checks
- Round-robin distribution
- Regional failover

**Cloud Load Balancers:**

**AWS ALB:**
```bash
aws elbv2 create-load-balancer \
    --name bitcell-lb \
    --subnets subnet-xxx subnet-yyy \
    --security-groups sg-xxx
```

**GCP Load Balancer:**
```bash
gcloud compute forwarding-rules create bitcell-lb \
    --global \
    --target-http-proxy=bitcell-proxy \
    --ports=8545
```

### Data Backup

**Automated backups:**

```bash
#!/bin/bash
# Backup script (run daily via cron)

BACKUP_DIR="/backups/bitcell"
DATA_DIR="/data/bitcell"
DATE=$(date +%Y%m%d)

# Create backup
tar -czf "$BACKUP_DIR/bitcell-$DATE.tar.gz" "$DATA_DIR"

# Upload to S3/GCS
aws s3 cp "$BACKUP_DIR/bitcell-$DATE.tar.gz" \
    s3://bitcell-backups/

# Clean old backups (keep last 30 days)
find "$BACKUP_DIR" -name "bitcell-*.tar.gz" -mtime +30 -delete
```

---

## Performance Tuning

### Node Optimization

**Increase file descriptor limits:**

```bash
# /etc/security/limits.conf
* soft nofile 65536
* hard nofile 65536
```

**Kernel parameters:**

```bash
# /etc/sysctl.conf
net.core.rmem_max = 134217728
net.core.wmem_max = 134217728
net.ipv4.tcp_rmem = 4096 87380 67108864
net.ipv4.tcp_wmem = 4096 65536 67108864
```

**Docker resource limits:**

```yaml
# docker-compose.yml
services:
  node-us-east-1:
    deploy:
      resources:
        limits:
          cpus: '8'
          memory: 16G
        reservations:
          cpus: '4'
          memory: 8G
```

### Database Tuning

**RocksDB settings (when implemented in RC2):**

```toml
# config.toml
[database]
max_open_files = 10000
write_buffer_size = 67108864  # 64MB
max_write_buffer_number = 3
```

---

## Troubleshooting

### Common Issues

**1. Nodes not connecting:**
```bash
# Check network
docker network inspect bitcell_bitcell-net

# Check bootstrap nodes
docker logs bitcell-node-us-east-1 | grep bootstrap

# Test connectivity
docker exec bitcell-node-us-east-1 ping node-us-west-1
```

**2. High latency:**
```bash
# Measure latency
for node in node-us-east-1 node-us-west-1 node-eu-central-1; do
    echo "Testing $node..."
    docker exec bitcell-node-us-east-1 ping -c 10 $node | tail -1
done
```

**3. Prometheus not scraping:**
```bash
# Check targets
curl http://localhost:9999/api/v1/targets | jq

# Verify metrics endpoint
curl http://localhost:9090/metrics
```

### Logs

**Docker Compose:**
```bash
docker-compose logs -f --tail=100 node-us-east-1
```

**Kubernetes:**
```bash
kubectl logs -f statefulset/bitcell-node-us-east \
    -n bitcell-production --tail=100
```

---

## Maintenance

### Rolling Updates

**Docker Compose:**
```bash
# Pull new image
docker pull bitcell-node:latest

# Update one node at a time
docker-compose stop node-us-east-1
docker-compose up -d node-us-east-1

# Wait for node to sync, then continue
sleep 60
docker-compose stop node-us-east-2
docker-compose up -d node-us-east-2
```

**Kubernetes:**
```bash
# Update image
kubectl set image statefulset/bitcell-node-us-east \
    bitcell-node=bitcell-node:latest \
    -n bitcell-production

# Monitor rollout
kubectl rollout status statefulset/bitcell-node-us-east \
    -n bitcell-production
```

### Database Maintenance

**Pruning (when available):**
```bash
docker exec bitcell-node-us-east-1 \
    bitcell-node prune --keep-recent 10000
```

---

## Security

### SSL/TLS

**For production, enable TLS:**

```yaml
# HAProxy TLS termination
frontend rpc_frontend
    bind *:443 ssl crt /etc/ssl/certs/bitcell.pem
```

### Authentication

**RPC Authentication:**
```bash
# Generate API key
API_KEY=$(openssl rand -hex 32)

# Configure node
docker-compose up -d -e RPC_API_KEY=$API_KEY node-us-east-1
```

### Secrets Management

**Use HashiCorp Vault or cloud KMS:**

```bash
# Store secret in Vault
vault kv put secret/bitcell/api-key value=$API_KEY

# Retrieve in startup script
API_KEY=$(vault kv get -field=value secret/bitcell/api-key)
```

---

## Cost Optimization

### Cloud Provider Estimates

**AWS (per month):**
- 7 EC2 instances (t3.xlarge): ~$1,200
- EBS storage (700GB): ~$70
- Data transfer: ~$200
- **Total: ~$1,500/month**

**GCP (per month):**
- 7 n2-standard-4 instances: ~$1,100
- Persistent SSD (700GB): ~$120
- Network egress: ~$150
- **Total: ~$1,400/month**

### Optimization Tips

1. Use spot/preemptible instances for non-critical nodes
2. Enable auto-scaling during low traffic
3. Use regional storage for backups
4. Implement caching where possible
5. Monitor and rightsize resources

---

## Support

- **Documentation**: https://docs.bitcell.network
- **Community**: https://discord.gg/bitcell
- **Issues**: https://github.com/Steake/BitCell/issues
- **Email**: support@bitcell.network
