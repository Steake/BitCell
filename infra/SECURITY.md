# Security Considerations for Production Deployment

## Overview

This document outlines security considerations and recommendations for deploying BitCell production infrastructure.

## Immediate Actions Required Before Production

### 1. Credentials and Secrets Management ⚠️ CRITICAL

**Issue**: Default credentials and placeholder secrets in configuration files.

**Files Affected**:
- `infra/docker/docker-compose.yml` - Grafana admin password
- `infra/monitoring/alertmanager.yml` - Slack webhook, PagerDuty keys

**Resolution**:

**Option A: Environment Variables (Recommended for Docker)**
```bash
# Set environment variables before starting
export GRAFANA_ADMIN_PASSWORD='your-secure-password-here'
export SLACK_API_URL='https://hooks.slack.com/services/YOUR/WEBHOOK'
export PAGERDUTY_SERVICE_KEY='your-pagerduty-key'

# Start with env vars
docker-compose up -d
```

**Option B: Docker Secrets (Recommended for Swarm)**
```bash
# Create secrets
echo 'your-secure-password' | docker secret create grafana_password -
echo 'your-webhook-url' | docker secret create slack_url -

# Reference in docker-compose.yml
secrets:
  - grafana_password
  - slack_url
```

**Option C: HashiCorp Vault (Recommended for Enterprise)**
```bash
# Store in Vault
vault kv put secret/bitcell/grafana password='secure-password'

# Retrieve in startup script
GRAFANA_PASSWORD=$(vault kv get -field=password secret/bitcell/grafana)
```

**Option D: Kubernetes Secrets (Recommended for K8s)**
```bash
# Create secrets
kubectl create secret generic bitcell-secrets \
    --from-literal=grafana-password='your-password' \
    --from-literal=slack-url='your-webhook' \
    --from-literal=pagerduty-key='your-key' \
    -n bitcell-production

# Reference in deployment.yaml (already configured)
env:
  - name: GF_SECURITY_ADMIN_PASSWORD
    valueFrom:
      secretKeyRef:
        name: bitcell-secrets
        key: grafana-password
```

### 2. HTTP Server Security ⚠️ IMPORTANT

**Issue**: Basic HTTP implementation in metrics server (lines 57-59 of metrics.rs).

**Current Implementation**:
- Basic string parsing
- No proper request validation
- May not handle malformed requests
- No rate limiting
- No authentication

**Short-term Mitigation**:
- Metrics endpoint exposed only on internal network
- Use firewall rules to restrict access
- Monitor for unusual activity

**Production Recommendation**:

Replace with production-grade HTTP server library:

```toml
# Cargo.toml
[dependencies]
axum = "0.7"
tower-http = "0.5"
```

```rust
// metrics.rs
use axum::{
    routing::get,
    Router,
    Json,
};
use tower_http::timeout::TimeoutLayer;
use std::time::Duration;

impl MetricsServer {
    pub async fn serve(self) -> Result<(), std::io::Error> {
        let app = Router::new()
            .route("/health", get(health_handler))
            .route("/metrics", get(metrics_handler))
            .layer(TimeoutLayer::new(Duration::from_secs(10)));
        
        let listener = tokio::net::TcpListener::bind(
            format!("0.0.0.0:{}", self.port)
        ).await?;
        
        axum::serve(listener, app).await?;
        Ok(())
    }
}
```

**Alternative Libraries**:
- **axum**: Modern, ergonomic, well-maintained
- **warp**: Fast, filter-based composition
- **actix-web**: Mature, high performance
- **tide**: Simple, beginner-friendly

### 3. Cross-Region Latency Monitoring ⚠️ MEDIUM

**Issue**: Using Prometheus scrape duration as proxy for latency (alerts.yml line 136).

**Current Alert**:
```yaml
- alert: HighCrossRegionLatency
  expr: scrape_duration_seconds{job=~"bitcell-.*"} > 0.2
```

**Problems**:
- Scrape duration affected by CPU load, memory, disk I/O
- Not a reliable indicator of network latency
- May cause false positives

**Production Recommendation**:

Implement proper latency metrics:

```rust
// Add to MetricsRegistry
pub struct MetricsRegistry {
    // ... existing fields
    cross_region_latency_ms: Arc<DashMap<String, AtomicU64>>,
}

impl MetricsRegistry {
    pub fn record_peer_latency(&self, peer_id: &str, latency_ms: u64) {
        self.cross_region_latency_ms
            .entry(peer_id.to_string())
            .or_insert(AtomicU64::new(0))
            .store(latency_ms, Ordering::Relaxed);
    }
    
    pub fn export_prometheus(&self) -> String {
        let mut output = String::new();
        
        // ... existing metrics
        
        // Add latency metrics
        output.push_str("# HELP bitcell_peer_latency_ms Peer-to-peer latency\n");
        output.push_str("# TYPE bitcell_peer_latency_ms gauge\n");
        for entry in self.cross_region_latency_ms.iter() {
            output.push_str(&format!(
                "bitcell_peer_latency_ms{{peer=\"{}\"}} {}\n",
                entry.key(),
                entry.value().load(Ordering::Relaxed)
            ));
        }
        
        output
    }
}
```

Update alert rule:
```yaml
- alert: HighCrossRegionLatency
  expr: bitcell_peer_latency_ms > 200
  for: 10m
  labels:
    severity: warning
  annotations:
    summary: "High latency to peer {{ $labels.peer }}"
    description: "Latency is {{ $value }}ms, exceeding 200ms target."
```

### 4. TLS/SSL Encryption 🔒 CRITICAL

**Current State**: All communication is unencrypted HTTP.

**Production Requirements**:

**A. RPC API Encryption**
```yaml
# haproxy.cfg
frontend rpc_frontend
    bind *:443 ssl crt /etc/ssl/certs/bitcell.pem
    redirect scheme https if !{ ssl_fc }
    
    # Force TLS 1.2+
    ssl-min-ver TLSv1.2
    
    # Strong ciphers only
    ssl-default-bind-ciphers ECDHE-ECDSA-AES128-GCM-SHA256:ECDHE-RSA-AES128-GCM-SHA256
```

**B. Grafana HTTPS**
```yaml
# docker-compose.yml
environment:
  - GF_SERVER_PROTOCOL=https
  - GF_SERVER_CERT_FILE=/etc/grafana/cert.pem
  - GF_SERVER_CERT_KEY=/etc/grafana/key.pem
```

**C. Mutual TLS for Node-to-Node**
```rust
// Configure libp2p with TLS
use libp2p::tls;

let transport = tcp::async_io::Transport::new(tcp::Config::default())
    .upgrade(upgrade::Version::V1)
    .authenticate(tls::Config::new(keypair)?)
    .multiplex(yamux::Config::default());
```

**Obtaining Certificates**:

**Option A: Let's Encrypt (Free)**
```bash
# Using certbot
certbot certonly --standalone -d bitcell.network -d *.bitcell.network
```

**Option B: Cloud Provider**
```bash
# AWS Certificate Manager
aws acm request-certificate --domain-name bitcell.network

# GCP Certificate Manager
gcloud certificate-manager certificates create bitcell-cert \
    --domains=bitcell.network
```

**Option C: Self-Signed (Development Only)**
```bash
# Generate self-signed cert
openssl req -x509 -newkey rsa:4096 \
    -keyout key.pem -out cert.pem \
    -days 365 -nodes
```

### 5. Network Security

**Firewall Configuration**:

```bash
# Allow only necessary ports
iptables -A INPUT -p tcp --dport 9000:9010 -j ACCEPT  # P2P
iptables -A INPUT -p tcp --dport 8545 -s 10.0.0.0/8 -j ACCEPT  # RPC (internal only)
iptables -A INPUT -p tcp --dport 9090:9100 -s 172.20.0.0/16 -j ACCEPT  # Metrics (monitoring only)
iptables -A INPUT -j DROP  # Drop everything else
```

**Cloud Security Groups**:

```bash
# AWS Security Group
aws ec2 authorize-security-group-ingress \
    --group-id sg-xxx \
    --protocol tcp --port 9000-9010 \
    --cidr 0.0.0.0/0  # P2P public
    
aws ec2 authorize-security-group-ingress \
    --group-id sg-xxx \
    --protocol tcp --port 8545 \
    --source-group sg-yyy  # RPC from load balancer only
```

**VPN for Admin Access**:
- Use WireGuard or OpenVPN for admin access
- Grafana/Prometheus/Alertmanager behind VPN only
- No public access to monitoring

### 6. Rate Limiting and DDoS Protection

**Implement Rate Limiting**:

```yaml
# haproxy.cfg
frontend rpc_frontend
    # Rate limit: 100 requests per 10 seconds per IP
    stick-table type ip size 100k expire 30s store http_req_rate(10s)
    http-request track-sc0 src
    http-request deny if { sc_http_req_rate(0) gt 100 }
```

**Use Cloud DDoS Protection**:
- AWS Shield
- GCP Cloud Armor
- Cloudflare
- Azure DDoS Protection

### 7. Audit Logging

**Enable Comprehensive Logging**:

```yaml
# docker-compose.yml
logging:
  driver: "json-file"
  options:
    max-size: "10m"
    max-file: "3"
    labels: "production,bitcell"
```

**Centralized Logging**:
- ELK Stack (Elasticsearch, Logstash, Kibana)
- Loki + Grafana
- Cloud provider logging (CloudWatch, Stackdriver)

**Log Everything**:
- Authentication attempts
- Configuration changes
- Alert acknowledgments
- Deployment events
- Security events

### 8. Access Control

**Implement RBAC**:

```yaml
# Grafana RBAC
apiVersion: 1
organizations:
  - name: BitCell
    orgId: 1
    role: Admin
    
users:
  - login: oncall-engineer
    role: Editor
  - login: readonly-viewer
    role: Viewer
```

**API Authentication**:

```rust
// Add API key validation
#[derive(Debug)]
struct ApiKeyAuth {
    valid_keys: HashSet<String>,
}

impl ApiKeyAuth {
    fn validate(&self, key: &str) -> bool {
        self.valid_keys.contains(key)
    }
}

// In request handler
if let Some(auth_header) = request.headers().get("Authorization") {
    if !auth.validate(auth_header.to_str()?) {
        return Err(AuthError::InvalidKey);
    }
}
```

### 9. Backup and Recovery

**Implement Automated Backups**:

```bash
#!/bin/bash
# /etc/cron.daily/bitcell-backup

DATE=$(date +%Y%m%d)
BACKUP_DIR="/backups/bitcell"

# Backup databases
for node in us-east-1 us-west-1 eu-central-1; do
    docker exec bitcell-$node tar -czf - /data/bitcell \
        > "$BACKUP_DIR/bitcell-$node-$DATE.tar.gz"
done

# Upload to S3/GCS
aws s3 sync "$BACKUP_DIR" s3://bitcell-backups/

# Retain last 30 days
find "$BACKUP_DIR" -mtime +30 -delete
```

**Test Recovery**:
- Monthly recovery drills
- Document RTO/RPO
- Validate backup integrity

### 10. Monitoring Security

**Monitor Security Metrics**:

```yaml
# alerts.yml
- alert: UnauthorizedAccess
  expr: rate(bitcell_auth_failures_total[5m]) > 10
  
- alert: UnusualTraffic
  expr: rate(bitcell_bytes_received_total[5m]) > 100000000  # 100MB/s
  
- alert: ConfigurationChanged
  expr: bitcell_config_changes_total > 0
```

## Security Checklist

Before going to production, verify:

- [ ] All default passwords changed
- [ ] Secrets stored securely (Vault/KMS/Secrets Manager)
- [ ] TLS/SSL enabled for all public endpoints
- [ ] Firewall rules configured
- [ ] VPN set up for admin access
- [ ] Rate limiting enabled
- [ ] DDoS protection active
- [ ] Audit logging enabled
- [ ] Backups automated and tested
- [ ] Security monitoring alerts configured
- [ ] Incident response plan tested
- [ ] Security audit completed
- [ ] Penetration testing performed

## Security Incident Response

If security breach detected:

1. **Isolate**: Disconnect affected systems
2. **Assess**: Determine scope and impact
3. **Contain**: Prevent further damage
4. **Eradicate**: Remove threat
5. **Recover**: Restore from clean backups
6. **Document**: Create incident report
7. **Improve**: Update security measures

**Emergency Contacts**:
- Security Team: security@bitcell.network
- On-Call: See PagerDuty
- External: [Your security partner/vendor]

## Compliance

Consider requirements for:
- GDPR (if handling EU user data)
- SOC 2 (for enterprise customers)
- ISO 27001 (information security)
- PCI DSS (if handling payments)

## Regular Security Maintenance

**Weekly**:
- Review security alerts
- Check for unauthorized access
- Verify backup completion

**Monthly**:
- Update dependencies
- Review firewall rules
- Test backup recovery
- Security patch review

**Quarterly**:
- Security audit
- Penetration testing
- Compliance review
- Update incident response plan

---

**Document Version**: 1.0
**Last Updated**: 2024-12-09
**Next Review**: 2024-12-16
