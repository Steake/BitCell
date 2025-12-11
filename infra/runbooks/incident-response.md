# BitCell Incident Response Runbooks

## Table of Contents

1. [On-Call Overview](#on-call-overview)
2. [Incident Severity Levels](#incident-severity-levels)
3. [Common Incidents](#common-incidents)
4. [Escalation Procedures](#escalation-procedures)

---

## On-Call Overview

### On-Call Responsibilities

- Monitor alerts from Alertmanager/PagerDuty
- Respond to incidents within SLA (see severity levels)
- Document all actions taken
- Escalate when necessary
- Conduct post-incident reviews

### On-Call Rotation

- **Primary On-Call**: First responder, 24/7 coverage
- **Secondary On-Call**: Backup for escalations
- **Week Duration**: Monday 9am to following Monday 9am
- **Handoff**: Monday morning with previous week's summary

### Tools Access

- **Monitoring**: Grafana (https://grafana.bitcell.network)
- **Alerts**: Alertmanager (https://alerts.bitcell.network)
- **Logs**: Centralized logging via Docker/K8s logs
- **Metrics**: Prometheus (https://prometheus.bitcell.network)

---

## Incident Severity Levels

### P0 - Critical (Response: Immediate, <15 min)

**Symptoms:**
- Complete network outage
- All regions down
- Data loss occurring
- Security breach

**Actions:**
1. Page entire team
2. Start incident war room
3. Begin immediate investigation
4. Notify leadership

---

### P1 - High (Response: <30 min)

**Symptoms:**
- Regional outage
- >30% nodes down
- Chain not progressing
- Major performance degradation

**Actions:**
1. Acknowledge alert
2. Begin investigation
3. Update status page
4. Escalate if not resolved in 1 hour

---

### P2 - Medium (Response: <2 hours)

**Symptoms:**
- Single node down
- High latency
- Minor service degradation
- Low peer counts

**Actions:**
1. Acknowledge alert
2. Investigate during business hours
3. Document findings

---

### P3 - Low (Response: Next business day)

**Symptoms:**
- Non-critical warnings
- Resource usage alerts
- Configuration issues

**Actions:**
1. Document in backlog
2. Address during maintenance window

---

## Common Incidents

### 1. Node Down

**Alert:** `NodeDown`

**Symptoms:**
- Node not responding to health checks
- No metrics from node
- Peer count decreasing on other nodes

**Diagnosis:**
```bash
# Check node status
docker ps | grep bitcell-node-<id>

# Check logs
docker logs bitcell-node-<id> --tail 100

# Check system resources
docker stats bitcell-node-<id>
```

**Resolution:**
```bash
# Restart node
docker-compose -f infra/docker/docker-compose.yml restart node-<id>

# Or in Kubernetes
kubectl rollout restart statefulset/bitcell-node-<region> -n bitcell-production

# Verify recovery
curl http://localhost:<metrics-port>/health
```

**Follow-up:**
- Monitor for 30 minutes
- Check if issue recurs
- Review logs for root cause

---

### 2. Regional Failure

**Alert:** `RegionDown` or `RegionDegraded`

**Symptoms:**
- All nodes in one region down
- Increased latency from that region
- Load shift to other regions

**Diagnosis:**
```bash
# Check all nodes in region
for node in node-us-east-1 node-us-east-2; do
    docker ps | grep $node
    echo "---"
done

# Check network connectivity
docker network inspect bitcell_bitcell-net

# Check cloud provider status
# Visit AWS/GCP/Azure status page for that region
```

**Resolution:**

**If cloud provider issue:**
1. Wait for provider resolution
2. Monitor other regions
3. Update status page
4. Consider manual failover if critical

**If configuration issue:**
```bash
# Restart all nodes in region
docker-compose -f infra/docker/docker-compose.yml restart \
    node-us-east-1 node-us-east-2

# Check bootstrap configuration
docker exec bitcell-node-us-east-1 cat /data/bitcell/config.toml
```

**Follow-up:**
- Ensure region fully recovered
- Verify cross-region replication
- Check for data consistency

---

### 3. Chain Not Progressing

**Alert:** `ChainNotProgressing`

**Symptoms:**
- Block height not increasing
- No new blocks for 10+ minutes
- Transactions not confirming

**Diagnosis:**
```bash
# Check chain height on multiple nodes
for port in 8545 8546 8547; do
    curl -X POST http://localhost:$port \
        -H "Content-Type: application/json" \
        -d '{"jsonrpc":"2.0","method":"eth_blockNumber","params":[],"id":1}'
done

# Check miner status
curl http://localhost:9090/metrics | grep bitcell_active_miners
```

**Resolution:**

**If no active miners:**
```bash
# Check EBSL trust system
curl http://localhost:9090/metrics | grep bitcell_banned_miners

# May need to manually unban miners or reset trust scores
# (requires admin access to state database)
```

**If network partition:**
```bash
# Check peer connectivity
curl http://localhost:9090/metrics | grep bitcell_peer_count

# Restart nodes to re-establish connections
docker-compose restart
```

**Follow-up:**
- Monitor block production rate
- Verify transaction processing
- Check for consensus issues

---

### 4. High Proof Generation Time

**Alert:** `HighProofGenerationTime`

**Symptoms:**
- Proof generation > 30 seconds
- Block delays
- Miner timeout warnings

**Diagnosis:**
```bash
# Check proof metrics
curl http://localhost:9090/metrics | grep proof_gen_time

# Check CPU/memory usage
docker stats --no-stream

# Check for resource contention
top -b -n 1 | head -20
```

**Resolution:**

**If resource constrained:**
```bash
# Scale up node resources (Docker)
docker update --cpus 8 --memory 16g bitcell-node-us-east-1

# Or in Kubernetes, update resource limits
kubectl edit statefulset bitcell-node-us-east -n bitcell-production
```

**If software issue:**
- Check for recent code changes
- Review proof generation logs
- Consider rolling back if needed

**Follow-up:**
- Monitor proof generation times
- Consider GPU acceleration
- Optimize proof circuits if pattern continues

---

### 5. No Peers / Network Isolation

**Alert:** `NoPeers` or `LowPeerCount`

**Symptoms:**
- Node has 0-1 connected peers
- Sync progress stopped
- Node isolated from network

**Diagnosis:**
```bash
# Check peer count
curl http://localhost:9090/metrics | grep peer_count

# Check network logs
docker logs bitcell-node-us-east-1 | grep -i peer

# Check bootstrap node connectivity
docker exec bitcell-node-us-east-1 ping -c 3 node-us-west-1

# Check firewall/security groups
iptables -L -n
# Or check cloud provider security groups
```

**Resolution:**
```bash
# Restart node to reconnect
docker-compose restart node-us-east-1

# Update bootstrap nodes if needed
docker-compose down node-us-east-1
# Edit docker-compose.yml BOOTSTRAP_NODES
docker-compose up -d node-us-east-1

# Check DHT
curl http://localhost:9090/metrics | grep dht_peer
```

**Follow-up:**
- Verify peer count stabilizes
- Check for network issues
- Review firewall rules

---

### 6. High Pending Transactions

**Alert:** `HighPendingTransactions`

**Symptoms:**
- >1000 pending transactions
- Transaction delays
- Mempool backlog

**Diagnosis:**
```bash
# Check pending tx count
curl http://localhost:9090/metrics | grep pending_txs

# Check transaction processing rate
curl http://localhost:9090/metrics | grep txs_processed_total
```

**Resolution:**

**If processing bottleneck:**
- Verify block production is normal
- Check for consensus issues
- Monitor proof generation

**If spam attack:**
- Implement rate limiting
- Adjust gas prices
- Consider mempool pruning

**Follow-up:**
- Monitor mempool size
- Adjust gas pricing if needed
- Review transaction patterns

---

### 7. Database Issues

**Symptoms:**
- Slow queries
- Disk space warnings
- State corruption errors

**Diagnosis:**
```bash
# Check disk usage
df -h /data/bitcell

# Check database size
du -sh /data/bitcell/db

# Check for corruption
docker exec bitcell-node-us-east-1 bitcell-node db-check
```

**Resolution:**

**If disk space low:**
```bash
# Enable pruning
docker exec bitcell-node-us-east-1 \
    bitcell-node prune --keep-recent 10000

# Or add more storage
# Resize volume in cloud provider
```

**If corruption:**
```bash
# Stop node
docker-compose stop node-us-east-1

# Restore from backup
cp -r /backups/bitcell-latest /data/bitcell

# Or resync from network
rm -rf /data/bitcell/db
docker-compose start node-us-east-1
```

---

### 8. Security Incidents

**Alert:** Custom security alerts or manual detection

**Symptoms:**
- Unusual transaction patterns
- Unauthorized access attempts
- Byzantine behavior detected

**Immediate Actions:**
1. **DO NOT PANIC** - Document everything
2. **Isolate** affected systems if actively exploited
3. **Page security team** immediately
4. **Preserve evidence** - take snapshots, save logs
5. **Assess impact** - what data/systems affected

**Investigation:**
```bash
# Review access logs
docker logs bitcell-node-us-east-1 | grep -i "error\|fail\|attack"

# Check recent commits/deployments
git log -10 --oneline

# Review firewall logs
iptables -L -v -n

# Check for anomalous transactions
# Use block explorer or RPC calls
```

**Resolution:**
- Follow security incident response plan
- Coordinate with security team
- May require emergency shutdown
- Prepare public disclosure if needed

---

## Escalation Procedures

### When to Escalate

- Unable to resolve P1 incident within 1 hour
- P0 incident (always escalate immediately)
- Security incident
- Unfamiliar situation
- Need additional expertise

### Escalation Contacts

**Primary Escalation:**
- Secondary On-Call Engineer
- Platform Lead

**Secondary Escalation:**
- Engineering Manager
- CTO

**Security Escalation:**
- Security Team Lead
- CISO

### Escalation Process

1. **Assess** severity and impact
2. **Document** what you've tried
3. **Call** next level (don't just message)
4. **Brief** them quickly on situation
5. **Handoff** or collaborate on resolution

---

## Post-Incident Review

### Required for P0/P1 Incidents

**Within 48 hours of resolution:**

1. **Timeline** - Document event sequence
2. **Root Cause** - Identify what happened and why
3. **Impact** - Quantify downtime, affected users
4. **Resolution** - What fixed it
5. **Action Items** - Preventive measures
6. **Follow-up** - Assign owners and deadlines

**Template:** See `incident-report-template.md`

---

## Emergency Contacts

- **Primary On-Call**: See PagerDuty schedule
- **Secondary On-Call**: See PagerDuty schedule
- **Platform Lead**: [contact info]
- **Security Team**: security@bitcell.network
- **Emergency Hotline**: [phone number]

---

## Additional Resources

- [Architecture Documentation](../../docs/ARCHITECTURE.md)
- [Deployment Guide](./deployment-guide.md)
- [Monitoring Dashboard](https://grafana.bitcell.network)
- [Status Page](https://status.bitcell.network)
