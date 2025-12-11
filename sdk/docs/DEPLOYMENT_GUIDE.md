# Smart Contract Deployment Guide

This guide covers deploying BitCell smart contracts to testnet and mainnet environments.

## Table of Contents

1. [Deployment Overview](#deployment-overview)
2. [Prerequisites](#prerequisites)
3. [Local Testnet Deployment](#local-testnet-deployment)
4. [Public Testnet Deployment](#public-testnet-deployment)
5. [Mainnet Deployment](#mainnet-deployment)
6. [Verification](#verification)
7. [Troubleshooting](#troubleshooting)

---

## Deployment Overview

BitCell contract deployment involves:
1. **Compilation** - Converting ZKASM to bytecode
2. **Gas Estimation** - Calculating deployment cost
3. **Transaction Creation** - Building deployment transaction
4. **Signing** - Signing with deployer's private key
5. **Submission** - Broadcasting to network
6. **Confirmation** - Waiting for block inclusion

---

## Prerequisites

### Required Tools

- BitCell SDK (this directory)
- Rust toolchain (1.82+)
- Deployer account with sufficient balance

### Generate Deployer Account

```bash
# Using BitCell wallet
cargo run -p bitcell-wallet -- generate-key

# Output:
# Address: 0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb8
# Private Key: 0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80
```

**⚠️ SECURITY WARNING:**
- Never share your private key
- Use environment variables for keys
- Consider hardware wallet for mainnet

### Fund Your Account

#### Local Testnet
Accounts are pre-funded in local testnet.

#### Public Testnet
Use the faucet:
```bash
curl -X POST https://faucet.testnet.bitcell.network/request \
  -H "Content-Type: application/json" \
  -d '{"address": "0xYOUR_ADDRESS"}'
```

#### Mainnet
Purchase CELL tokens or receive from another account.

---

## Local Testnet Deployment

### Step 1: Start Local Testnet

```bash
cd sdk/
./tools/start-testnet.sh
```

Keep this running in a separate terminal.

### Step 2: Compile Contract

```bash
./tools/compile-contract.sh templates/token.zkasm
```

Output:
```
🔧 BitCell Contract Compiler
============================
Input:  templates/token.zkasm
Output: templates/token.bin

✨ Compilation Successful!
```

### Step 3: Deploy

```bash
export DEPLOYER_PRIVATE_KEY="0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80"
./tools/deploy-contract.sh templates/token.zkasm local
```

Output:
```
📦 BitCell Contract Deployment
==============================
Contract: templates/token.zkasm
Network:  local
RPC endpoint: http://127.0.0.1:8545

...

✨ Deployment Complete!
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

Contract Address: 0x5FbDB2315678afecb367f032d93F642f64180aa3
```

### Step 4: Verify

```bash
./tools/test-contract.sh 0x5FbDB2315678afecb367f032d93F642f64180aa3
```

---

## Public Testnet Deployment

### Network Information

- **RPC URL:** `https://testnet-rpc.bitcell.network`
- **Chain ID:** 99999
- **Block Explorer:** `https://testnet-explorer.bitcell.network`

### Deployment Steps

#### 1. Get Testnet Tokens

Request from faucet (see Prerequisites).

#### 2. Check Balance

```bash
curl -X POST https://testnet-rpc.bitcell.network \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "method": "eth_getBalance",
    "params": ["0xYOUR_ADDRESS", "latest"],
    "id": 1
  }'
```

#### 3. Deploy

```bash
export DEPLOYER_PRIVATE_KEY="0xYOUR_PRIVATE_KEY"
./tools/deploy-contract.sh templates/token.zkasm testnet
```

#### 4. Verify on Explorer

Visit: `https://testnet-explorer.bitcell.network/address/0xYOUR_CONTRACT`

---

## Mainnet Deployment

### ⚠️ Pre-Deployment Checklist

Before deploying to mainnet:

- [ ] Contract thoroughly tested on testnet
- [ ] Security audit completed (for significant contracts)
- [ ] Gas costs calculated and acceptable
- [ ] Sufficient CELL tokens for deployment
- [ ] Private key secured (hardware wallet recommended)
- [ ] Contract verified on explorer
- [ ] Emergency procedures documented
- [ ] Upgrade mechanism considered (if needed)

### Network Information

- **RPC URL:** `https://rpc.bitcell.network`
- **Chain ID:** 9999
- **Block Explorer:** `https://explorer.bitcell.network`

### Deployment Steps

#### 1. Final Testing

Run comprehensive tests on testnet first:

```bash
# Deploy to testnet
./tools/deploy-contract.sh my_contract.zkasm testnet

# Run extensive tests
./tools/test-contract.sh 0xTESTNET_CONTRACT_ADDRESS

# Monitor for 24-48 hours
```

#### 2. Prepare Mainnet Account

```bash
# Check balance (should have enough for gas + safety margin)
curl -X POST https://rpc.bitcell.network \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "method": "eth_getBalance",
    "params": ["0xYOUR_ADDRESS", "latest"],
    "id": 1
  }'
```

#### 3. Deploy to Mainnet

```bash
export DEPLOYER_PRIVATE_KEY="0xYOUR_MAINNET_KEY"
./tools/deploy-contract.sh my_contract.zkasm mainnet
```

#### 4. Verify Deployment

```bash
# Check contract exists
curl -X POST https://rpc.bitcell.network \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "method": "eth_getCode",
    "params": ["0xYOUR_CONTRACT_ADDRESS", "latest"],
    "id": 1
  }'

# Should return bytecode, not "0x"
```

#### 5. Announce Deployment

Document your contract:
- Contract address
- Source code (on GitHub)
- Deployment transaction
- Interface ABI (if applicable)
- Security audit report (if any)

---

## Verification

### Verify Bytecode Match

After deployment, verify the bytecode matches your compilation:

```bash
# Get deployed bytecode
curl -X POST $RPC_URL \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "method": "eth_getCode",
    "params": ["0xCONTRACT_ADDRESS", "latest"],
    "id": 1
  }' | jq -r '.result'

# Compare with local compilation
sha256sum templates/token.bin
```

### Verify State

Check initial contract state:

```bash
# Call a view function
curl -X POST $RPC_URL \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "method": "eth_call",
    "params": [{
      "to": "0xCONTRACT_ADDRESS",
      "data": "0x..."
    }, "latest"],
    "id": 1
  }'
```

---

## Gas Estimation

### Estimate Deployment Cost

```bash
# Rough estimate based on bytecode size
BYTECODE_SIZE=$(wc -c < templates/token.bin)
DEPLOYMENT_GAS=$((21000 + BYTECODE_SIZE * 200))
echo "Estimated gas: $DEPLOYMENT_GAS"

# Check current gas price
curl -X POST $RPC_URL \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "method": "eth_gasPrice",
    "id": 1
  }'

# Calculate cost
# Cost = Gas * GasPrice
```

### Gas Optimization Tips

1. **Minimize bytecode size** - Remove unnecessary code
2. **Optimize memory layout** - Use compact structures
3. **Batch operations** - Combine multiple updates
4. **Use cheaper instructions** - Prefer ADD over MUL when possible

---

## Troubleshooting

### Common Issues

#### 1. Insufficient Gas

**Error:** Transaction reverted due to insufficient gas

**Solution:**
- Increase gas limit in deployment transaction
- Optimize contract to use less gas

#### 2. Nonce Mismatch

**Error:** Invalid nonce

**Solution:**
```bash
# Get current nonce
curl -X POST $RPC_URL \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "method": "eth_getTransactionCount",
    "params": ["0xYOUR_ADDRESS", "latest"],
    "id": 1
  }'
```

#### 3. Insufficient Balance

**Error:** Insufficient funds for gas * price + value

**Solution:**
- Fund your account with more CELL tokens
- Reduce gas price (will take longer to confirm)

#### 4. Contract Already Deployed

**Error:** Contract already exists at address

**Solution:**
- Use a different nonce
- Deploy with different account
- Verify if existing contract is yours

#### 5. RPC Connection Issues

**Error:** Connection refused or timeout

**Solution:**
- Check network connectivity
- Verify RPC URL is correct
- Try alternative RPC endpoint
- Check if node is synced

---

## Security Considerations

### Before Deployment

1. **Audit your contract** - Consider professional audit for valuable contracts
2. **Test extensively** - Cover all edge cases
3. **Review gas usage** - Ensure operations can't exceed block gas limit
4. **Check access controls** - Verify authorization logic
5. **Consider upgradability** - Plan for future fixes if needed

### During Deployment

1. **Use secure key management** - Hardware wallet for mainnet
2. **Verify transaction details** - Double-check before signing
3. **Monitor deployment** - Watch for confirmation
4. **Test immediately** - Verify contract works as expected

### After Deployment

1. **Document thoroughly** - Keep records of deployment
2. **Monitor activity** - Watch for unexpected behavior
3. **Prepare emergency response** - Have a plan for issues
4. **Communicate with users** - Announce any important changes

---

## Additional Resources

- **BitCell RPC API:** See `docs/RPC_API_Spec.md`
- **Security Best Practices:** See `BEST_PRACTICES.md`
- **Example Contracts:** See `templates/` and `examples/`
- **Community Support:** GitHub Discussions

---

**Last Updated:** December 2025  
**Version:** 1.0
