# Security Vulnerability Tracking Template

**Project:** BitCell Blockchain  
**Created:** December 2025  
**Status:** Active Tracking

---

## Vulnerability Entry Template

Use this template for each security finding discovered during the audit process.

```markdown
## Finding: [Brief Title]

**ID:** BITCELL-YYYY-NNN  
**Date Reported:** YYYY-MM-DD  
**Reporter:** [Name/Organization]  
**Severity:** [Critical / High / Medium / Low]  
**CVSS Score:** [0.0-10.0]  
**Status:** [Open / In Progress / Resolved / Accepted Risk / Wont Fix]  
**Assignee:** [Developer Name]

### Affected Components
- **Crate:** bitcell-[component]
- **File:** path/to/file.rs
- **Function/Module:** specific_function
- **Lines:** start-end

### Description
[Detailed description of the vulnerability, including how it manifests and under what conditions]

### Impact
**Confidentiality:** [None / Low / Medium / High]  
**Integrity:** [None / Low / Medium / High]  
**Availability:** [None / Low / Medium / High]  

[Detailed explanation of the potential impact if exploited]

### Attack Scenario
[Step-by-step description of how an attacker could exploit this vulnerability]

1. Attacker does X
2. System responds with Y
3. Attacker leverages Y to achieve Z

### Proof of Concept
```rust
// Code demonstrating the vulnerability
fn exploit_example() {
    // PoC code here
}
```

### Root Cause Analysis
[Technical explanation of why the vulnerability exists]

### Remediation
**Recommended Fix:**
```rust
// Proposed code fix
fn secure_implementation() {
    // Fixed code here
}
```

**Alternative Solutions:**
1. [Alternative approach 1]
2. [Alternative approach 2]

### Verification
**Test Case:**
```rust
#[test]
fn test_vulnerability_fixed() {
    // Test to verify the fix
}
```

### Timeline
- **Discovered:** YYYY-MM-DD
- **Acknowledged:** YYYY-MM-DD
- **Fix Developed:** YYYY-MM-DD
- **Fix Tested:** YYYY-MM-DD
- **Fix Deployed:** YYYY-MM-DD
- **Verified:** YYYY-MM-DD

### References
- [Link to related issues]
- [Link to CVE if applicable]
- [Link to relevant documentation]

### Notes
[Additional context, workarounds, or information]
```

---

## Known Vulnerabilities (From Repository Memories)

### BITCELL-2025-001: Faucet TOCTOU Race Condition

**ID:** BITCELL-2025-001  
**Date Reported:** 2025-12-09  
**Severity:** Medium  
**CVSS Score:** 5.9  
**Status:** Open  

**Affected Components:**
- Crate: bitcell-admin
- File: crates/bitcell-admin/src/faucet.rs
- Lines: 285-313

**Description:**
Time-of-check-time-of-use (TOCTOU) race condition between rate limit check and request recording. Multiple concurrent requests from the same address can bypass rate limits.

**Impact:**
- **Confidentiality:** None
- **Integrity:** Low (faucet drainage)
- **Availability:** Medium (DoS via fund depletion)

Attacker can drain testnet faucet funds faster than intended rate limits allow.

**Remediation:**
Use atomic operations or locking to ensure check and record happen atomically.

```rust
// Use RwLock properly or atomic compare-and-swap
let mut rate_limits = self.rate_limits.write().await;
if !self.check_rate_limit_locked(&rate_limits, address) {
    return Err(Error::RateLimited);
}
self.record_request_locked(&mut rate_limits, address, amount);
```

---

### BITCELL-2025-002: Faucet CAPTCHA Placeholder

**ID:** BITCELL-2025-002  
**Date Reported:** 2025-12-09  
**Severity:** Medium  
**CVSS Score:** 5.3  
**Status:** Open

**Affected Components:**
- Crate: bitcell-admin
- File: crates/bitcell-admin/src/faucet.rs
- Lines: 266-282

**Description:**
CAPTCHA validation is placeholder-only and accepts any non-empty string. Provides no actual anti-abuse protection.

**Impact:**
- **Confidentiality:** None
- **Integrity:** None
- **Availability:** Medium (automated abuse)

Bots can easily bypass CAPTCHA checks, enabling automated faucet abuse.

**Remediation:**
Integrate real CAPTCHA service (hCaptcha, reCAPTCHA) or implement proof-of-work challenge.

---

### BITCELL-2025-003: Faucet Unbounded Memory Growth

**ID:** BITCELL-2025-003  
**Date Reported:** 2025-12-09  
**Severity:** Medium  
**CVSS Score:** 6.2  
**Status:** Open

**Affected Components:**
- Crate: bitcell-admin
- File: crates/bitcell-admin/src/faucet.rs
- Lines: 97-98, 325

**Description:**
Faucet `request_history` Vec and `rate_limits` HashMap grow unbounded without cleanup. No rotation mechanism like audit logger's 10k limit.

**Impact:**
- **Confidentiality:** None
- **Integrity:** None
- **Availability:** High (memory exhaustion)

Long-running faucet service will eventually exhaust memory causing crash.

**Remediation:**
Implement periodic cleanup of old entries:

```rust
// Add TTL-based cleanup
const MAX_HISTORY_SIZE: usize = 10_000;
const MAX_RATE_LIMIT_ENTRIES: usize = 100_000;
const RATE_LIMIT_TTL_SECS: u64 = 86400; // 24 hours

fn cleanup_old_entries(&mut self) {
    // Rotate history
    if self.request_history.len() > MAX_HISTORY_SIZE {
        self.request_history.drain(0..1000);
    }
    
    // Remove stale rate limit entries
    let cutoff = current_time() - RATE_LIMIT_TTL_SECS;
    self.rate_limits.retain(|_, entry| entry.last_request > cutoff);
}
```

---

### BITCELL-2025-004: Token Revocation Memory Leak

**ID:** BITCELL-2025-004  
**Date Reported:** 2025-12-09  
**Severity:** Low  
**CVSS Score:** 3.7  
**Status:** Open

**Affected Components:**
- Crate: bitcell-admin
- File: crates/bitcell-admin/src/auth.rs
- Lines: 99, 181-182, 204, 225

**Description:**
Token revocation uses in-memory HashSet without expiration cleanup. Revoked tokens accumulate indefinitely.

**Impact:**
- **Confidentiality:** None
- **Integrity:** None
- **Availability:** Low (slow memory leak)

Over time, revoked token set grows without bound causing slow memory leak.

**Remediation:**
Add TTL-based cleanup for expired tokens:

```rust
struct RevokedToken {
    token_hash: String,
    revoked_at: u64,
    expires_at: u64,
}

// Cleanup expired tokens periodically
fn cleanup_expired_revocations(&mut self) {
    let now = current_time();
    self.revoked_tokens.retain(|token| token.expires_at > now);
}
```

---

### BITCELL-2025-005: RBAC Enforcement Not Automatic

**ID:** BITCELL-2025-005  
**Date Reported:** 2025-12-09  
**Severity:** High  
**CVSS Score:** 7.5  
**Status:** Open

**Affected Components:**
- Crate: bitcell-admin
- File: crates/bitcell-admin/src/lib.rs, src/auth.rs
- Lines: Various handler functions

**Description:**
JWT middleware validates tokens but does NOT enforce role checks. Handlers must explicitly check roles using `user.claims.role.can_perform(Role::X)`. Easy to forget in new endpoints.

**Impact:**
- **Confidentiality:** High (unauthorized access to admin data)
- **Integrity:** High (unauthorized operations)
- **Availability:** Low

Missing role checks in handlers allow privilege escalation.

**Attack Scenario:**
1. Attacker obtains valid Viewer token
2. Attacker calls admin-only endpoint (e.g., node start/stop)
3. If handler forgot role check, operation succeeds
4. Attacker gains admin privileges

**Remediation:**
Create role-checking middleware or decorators:

```rust
// Add role requirement to route registration
.route("/api/admin/nodes/start", 
    post(start_node_handler).layer(RequireRole::Admin))
.route("/api/admin/metrics", 
    get(get_metrics_handler).layer(RequireRole::Operator))
```

---

### BITCELL-2025-006: WebSocket Subscription Memory Leak

**ID:** BITCELL-2025-006  
**Date Reported:** 2025-12-09  
**Severity:** Medium  
**CVSS Score:** 5.9  
**Status:** Open

**Affected Components:**
- Crate: bitcell-node
- File: crates/bitcell-node/src/ws.rs
- Lines: 123-138

**Description:**
WebSocket subscription broadcast silently ignores failed sends with `let _ = tx.send()`. No cleanup mechanism for closed client channels in SubscriptionManager.

**Impact:**
- **Confidentiality:** None
- **Integrity:** None
- **Availability:** Medium (memory leak from dead subscriptions)

Disconnected clients remain in subscription list, leaking memory over time.

**Remediation:**
Check send results and remove failed subscriptions:

```rust
// Track and remove dead subscriptions
self.subscriptions.retain(|sub_id, tx| {
    match tx.send(event.clone()) {
        Ok(_) => true,  // Keep active subscription
        Err(_) => {
            log::debug!("Removing dead subscription: {}", sub_id);
            false  // Remove failed subscription
        }
    }
});
```

---

## Vulnerability Statistics

| Severity | Open | In Progress | Resolved | Accepted | Total |
|----------|------|-------------|----------|----------|-------|
| Critical | 0    | 0           | 0        | 0        | 0     |
| High     | 1    | 0           | 0        | 0        | 1     |
| Medium   | 4    | 0           | 0        | 0        | 4     |
| Low      | 1    | 0           | 0        | 0        | 1     |
| **Total**| **6**| **0**       | **0**    | **0**    | **6** |

---

## Severity Classification Reference

### Critical (CVSS 9.0-10.0)
- Remote code execution
- Complete system compromise
- Private key extraction
- Consensus breaking
- Mass fund theft

### High (CVSS 7.0-8.9)
- Authentication bypass
- Privilege escalation
- Targeted fund theft
- Service disruption (DoS)
- Significant data breach

### Medium (CVSS 4.0-6.9)
- Information disclosure
- Limited DoS
- Protocol violations
- Resource leaks
- Missing security features

### Low (CVSS 0.1-3.9)
- Informational findings
- Best practice violations
- Code quality issues
- Minor misconfigurations
- Theoretical attacks

---

## Next Steps

1. **Immediate Actions:**
   - Review and validate all findings above
   - Prioritize fixes for High severity issues
   - Create GitHub issues for tracking

2. **Short-term (Before RC3):**
   - Fix all High severity issues
   - Fix critical Medium severity issues
   - Add security tests for fixes

3. **External Audit Preparation:**
   - Document all known issues
   - Prepare mitigation evidence
   - Ready codebase for audit

4. **Ongoing:**
   - Regular security reviews
   - Bug bounty program
   - Security awareness training

---

**Last Updated:** 2025-12-09  
**Next Review:** Before RC3 Release
