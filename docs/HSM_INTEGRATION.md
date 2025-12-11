# HSM Provider Integration Guide

This guide explains how to integrate and use Hardware Security Module (HSM) providers in BitCell for secure key management and transaction signing.

## Overview

BitCell supports multiple HSM providers for production-grade key security:

- **HashiCorp Vault Transit** - Enterprise secrets management
- **AWS CloudHSM / KMS** - AWS-native HSM solution
- **Azure Key Vault** - Azure-native managed HSM
- **Mock HSM** - Testing and development

All HSM operations are logged via the audit trail for compliance and security monitoring.

## Features

All HSM backends provide:
- ✅ ECDSA secp256k1 key generation
- ✅ Cryptographic signing operations
- ✅ Public key retrieval
- ✅ Key enumeration
- ✅ Audit logging
- ✅ Async/await API

## Building with HSM Support

HSM providers are behind feature flags to minimize dependencies:

```bash
# Build with Vault support
cargo build --features vault

# Build with AWS support  
cargo build --features aws-hsm

# Build with Azure support
cargo build --features azure-hsm

# Build with all HSM providers
cargo build --features vault,aws-hsm,azure-hsm
```

## HashiCorp Vault Transit

### Prerequisites

1. Running Vault server (dev or production)
2. Transit secrets engine enabled
3. Valid authentication token
4. Network access to Vault

### Setup Vault

```bash
# Start Vault dev server (for testing)
vault server -dev

# Enable transit engine
vault secrets enable transit

# Create a policy (production)
vault policy write bitcell-hsm - <<EOF
path "transit/keys/bitcell-*" {
  capabilities = ["create", "read", "list"]
}
path "transit/sign/bitcell-*" {
  capabilities = ["update"]
}
path "transit/verify/bitcell-*" {
  capabilities = ["update"]
}
EOF

# Create a token
vault token create -policy=bitcell-hsm
```

### Usage

```rust
use bitcell_admin::hsm::{HsmClient, HsmConfig};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Configure Vault
    let config = HsmConfig::vault(
        "http://127.0.0.1:8200",  // Vault address
        "s.xyz...",                // Vault token
        "bitcell-validator-key"    // Key name
    );
    
    // Connect to HSM
    let hsm = HsmClient::connect(config).await?;
    
    // Generate a new key
    let public_key = hsm.generate_key("bitcell-validator-key").await?;
    println!("Generated key: {:?}", public_key);
    
    // Sign a transaction hash
    let hash = bitcell_crypto::Hash256::hash(b"transaction data");
    let signature = hsm.sign(&hash).await?;
    
    // Verify signature
    assert!(signature.verify(&public_key, hash.as_bytes()).is_ok());
    
    // List all keys
    let keys = hsm.list_keys().await?;
    println!("Available keys: {:?}", keys);
    
    // Check audit log
    let audit = hsm.audit_log().await;
    for entry in audit {
        println!("{:?}", entry);
    }
    
    Ok(())
}
```

### Vault Configuration Options

```rust
let mut config = HsmConfig::vault(
    "https://vault.example.com",
    "s.token",
    "key-name"
);

// Customize settings
config.timeout_secs = 60;          // Increase timeout
config.audit_logging = true;       // Enable audit logging (default: true)
```

## AWS CloudHSM / KMS

### Prerequisites

1. AWS account with KMS enabled
2. IAM credentials (access key + secret key)
3. Appropriate IAM permissions
4. Network access to AWS KMS endpoint

### IAM Policy

```json
{
  "Version": "2012-10-17",
  "Statement": [
    {
      "Effect": "Allow",
      "Action": [
        "kms:CreateKey",
        "kms:CreateAlias",
        "kms:DescribeKey",
        "kms:GetPublicKey",
        "kms:Sign",
        "kms:ListAliases",
        "kms:ListKeys"
      ],
      "Resource": "*"
    }
  ]
}
```

### Usage

```rust
use bitcell_admin::hsm::{HsmClient, HsmConfig};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Configure AWS KMS
    let config = HsmConfig::aws(
        "kms.us-east-1.amazonaws.com",    // KMS endpoint
        "AKIAIOSFODNN7EXAMPLE",           // AWS access key
        "wJalr...EXAMPLEKEY",             // AWS secret key
        "bitcell-validator-key"            // Key alias
    );
    
    // Connect to HSM
    let hsm = HsmClient::connect(config).await?;
    
    // Generate a new key (creates key + alias)
    let public_key = hsm.generate_key("bitcell-validator-key").await?;
    
    // Sign with the key
    let hash = bitcell_crypto::Hash256::hash(b"transaction data");
    let signature = hsm.sign(&hash).await?;
    
    Ok(())
}
```

### Multi-Region Setup

```rust
// Different regions for high availability
let us_east = HsmConfig::aws("kms.us-east-1.amazonaws.com", ...);
let eu_west = HsmConfig::aws("kms.eu-west-1.amazonaws.com", ...);
let ap_south = HsmConfig::aws("kms.ap-south-1.amazonaws.com", ...);
```

### AWS Environment Variables

Alternatively, use environment variables:

```bash
export AWS_ACCESS_KEY_ID=AKIAIOSFODNN7EXAMPLE
export AWS_SECRET_ACCESS_KEY=wJalr...EXAMPLEKEY
export AWS_REGION=us-east-1
```

## Azure Key Vault

### Prerequisites

1. Azure subscription
2. Key Vault resource created
3. Service Principal with appropriate permissions
4. Client ID and Client Secret

### Setup Azure Key Vault

```bash
# Create resource group
az group create --name bitcell-rg --location eastus

# Create Key Vault
az keyvault create \
  --name bitcell-kv \
  --resource-group bitcell-rg \
  --location eastus

# Create service principal
az ad sp create-for-rbac \
  --name bitcell-hsm-sp \
  --role "Key Vault Crypto Officer" \
  --scopes /subscriptions/{subscription-id}/resourceGroups/bitcell-rg

# Note the appId (client ID), password (client secret), and tenant
```

### Usage

```rust
use bitcell_admin::hsm::{HsmClient, HsmConfig};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Configure Azure Key Vault
    let config = HsmConfig::azure(
        "https://bitcell-kv.vault.azure.net",  // Key Vault URL
        "tenant-id-guid",                       // Azure AD tenant ID
        "client-id-guid",                       // Service Principal client ID
        "client-secret-string",                 // Service Principal secret
        "bitcell-validator-key"                 // Key name
    );
    
    // Connect to HSM
    let hsm = HsmClient::connect(config).await?;
    
    // Generate a new key
    let public_key = hsm.generate_key("bitcell-validator-key").await?;
    
    // Sign with the key
    let hash = bitcell_crypto::Hash256::hash(b"transaction data");
    let signature = hsm.sign(&hash).await?;
    
    Ok(())
}
```

### Azure RBAC Roles

Required roles for the service principal:
- `Key Vault Crypto Officer` - Full crypto operations
- `Key Vault Crypto User` - Sign and verify only (read-only)

### Key Rotation

Azure Key Vault supports native key rotation:

```bash
# Rotate a key (creates new version)
az keyvault key rotate \
  --vault-name bitcell-kv \
  --name bitcell-validator-key

# Set rotation policy (e.g., rotate every 90 days)
az keyvault key rotation-policy update \
  --vault-name bitcell-kv \
  --name bitcell-validator-key \
  --value '{"lifetimeActions":[{"trigger":{"timeAfterCreate":"P90D"},"action":{"type":"Rotate"}}]}'
```

**Important Notes on Key Rotation:**

When a key is rotated, Azure Key Vault creates a new version while preserving all previous versions. This means:
- **New signatures** are created with the latest key version
- **Old signatures** remain valid and can be verified using their original key version
- The HSM client automatically uses the latest version for new signing operations
- Previous key versions remain accessible for signature verification

Example workflow:
1. Key v1 is used to sign transactions in January
2. Key is rotated → Key v2 is created in April
3. New transactions are signed with v2
4. Old transactions signed with v1 can still be verified using v1

This ensures backward compatibility and doesn't invalidate existing signatures.

The HSM client automatically uses the latest key version.

## Mock HSM (Testing)

For development and testing without real HSM infrastructure:

```rust
use bitcell_admin::hsm::{HsmClient, HsmConfig};

#[tokio::test]
async fn test_signing() {
    let config = HsmConfig::mock("test-key");
    let hsm = HsmClient::connect(config).await.unwrap();
    
    let public_key = hsm.generate_key("test-key").await.unwrap();
    let hash = bitcell_crypto::Hash256::hash(b"test");
    let signature = hsm.sign(&hash).await.unwrap();
    
    assert!(signature.verify(&public_key, hash.as_bytes()).is_ok());
}
```

## Audit Logging

All HSM operations are automatically logged:

```rust
let hsm = HsmClient::connect(config).await?;

// Perform operations
hsm.generate_key("key1").await?;
hsm.sign(&hash).await?;

// Retrieve audit log
let audit_entries = hsm.audit_log().await;
for entry in audit_entries {
    println!("[{}] {} on {} - {}",
        entry.timestamp,
        entry.operation,
        entry.key_name,
        if entry.success { "SUCCESS" } else { "FAILED" }
    );
}

// Clear audit log if needed
hsm.clear_audit_log().await;
```

Audit log entries include:
- Timestamp (Unix epoch)
- Operation type (generate_key, sign, get_public_key)
- Key name
- Success/failure status
- Error message (if failed)

The audit log is bounded to 10,000 entries with automatic rotation.

## Production Best Practices

### Security

1. **Never log credentials** - Credentials are automatically zeroed on drop
2. **Use separate keys per environment** - dev, staging, production
3. **Rotate keys regularly** - Follow HSM provider's rotation policies
4. **Monitor audit logs** - Set up alerts for suspicious activity
5. **Use mTLS** - Enable mutual TLS for Vault connections in production

### High Availability

1. **Multiple HSM instances** - Deploy across availability zones
2. **Failover logic** - Implement automatic failover between HSM providers
3. **Health checks** - Use `is_available()` for readiness probes
4. **Connection pooling** - Reuse HSM client instances

### Key Management

1. **Key naming convention** - Use prefixes: `bitcell-{env}-{purpose}-key`
2. **Backup strategies** - Export public keys, never private keys
3. **Access control** - Principle of least privilege
4. **Compliance** - Document key lifecycle for audits

### Example Production Configuration

```rust
use std::time::Duration;
use tokio::time::timeout;

async fn create_production_hsm() -> Result<HsmClient, Box<dyn std::error::Error>> {
    let config = HsmConfig::vault(
        std::env::var("VAULT_ADDR")?,
        std::env::var("VAULT_TOKEN")?,
        "bitcell-prod-validator-key"
    );
    
    // Add timeout for connection
    let hsm = timeout(
        Duration::from_secs(30),
        HsmClient::connect(config)
    ).await??;
    
    // Verify connectivity
    if !hsm.is_available().await {
        return Err("HSM not available".into());
    }
    
    Ok(hsm)
}
```

## Troubleshooting

### Vault Connection Issues

```
Error: HSM connection failed: Cannot connect to Vault
```

- Check Vault server is running: `vault status`
- Verify network connectivity: `curl $VAULT_ADDR/v1/sys/health`
- Check token is valid: `vault token lookup`
- Ensure transit engine is mounted: `vault secrets list`

### AWS KMS Permission Errors

```
Error: HSM internal error: Failed to create key: AccessDeniedException
```

- Verify IAM credentials are correct
- Check IAM policy includes required KMS actions
- Ensure KMS endpoint is accessible from your network
- Verify AWS region is correct

### Azure Key Vault Authentication

```
Error: HSM authentication failed
```

- Verify service principal credentials
- Check Key Vault access policies or RBAC assignments
- Ensure Key Vault firewall allows your IP
- Verify vault URL format: `https://{vault-name}.vault.azure.net`

### Signature Verification Failures

```
Error: Invalid signature
```

- Ensure using correct public key for verification
- Check hash algorithm matches (SHA-256)
- Verify signature format is compatible with BitCell
- For AWS/Azure: DER encoding may need conversion

## API Reference

### HsmConfig Methods

```rust
// Create configs
HsmConfig::vault(endpoint, token, key_name) -> HsmConfig
HsmConfig::aws(endpoint, access_key, secret_key, key_name) -> HsmConfig
HsmConfig::azure(vault_url, client_id, client_secret, key_name) -> HsmConfig
HsmConfig::mock(key_name) -> HsmConfig
```

### HsmClient Methods

```rust
// Connection
HsmClient::connect(config: HsmConfig) -> Result<HsmClient>

// Operations
hsm.is_available() -> bool
hsm.generate_key(key_name: &str) -> Result<PublicKey>
hsm.get_public_key() -> Result<PublicKey>
hsm.get_public_key_by_name(key_name: &str) -> Result<PublicKey>
hsm.sign(hash: &Hash256) -> Result<Signature>
hsm.sign_with_key(key_name: &str, hash: &Hash256) -> Result<Signature>
hsm.list_keys() -> Result<Vec<String>>

// Audit
hsm.audit_log() -> Vec<AuditEntry>
hsm.clear_audit_log()
```

## Testing

Run HSM tests:

```bash
# Run all tests (mock HSM only)
cargo test --package bitcell-admin --lib hsm

# Run with Vault (requires running Vault instance)
cargo test --package bitcell-admin --lib hsm::vault --features vault -- --ignored

# Run with AWS (requires AWS credentials)
cargo test --package bitcell-admin --lib hsm::aws --features aws-hsm -- --ignored

# Run with Azure (requires Azure credentials)
cargo test --package bitcell-admin --lib hsm::azure --features azure-hsm -- --ignored
```

## Support

For issues or questions:
- GitHub Issues: https://github.com/Steake/BitCell/issues
- Documentation: https://github.com/Steake/BitCell/docs
- Security: See SECURITY.md for responsible disclosure

## License

See LICENSE file in the repository root.
