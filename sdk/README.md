# BitCell Smart Contract SDK

Welcome to the BitCell Smart Contract SDK! This toolkit provides everything you need to develop, test, and deploy smart contracts on the BitCell blockchain.

## 🚀 Quick Start

```bash
# 1. Navigate to SDK directory
cd sdk/

# 2. Start local testnet
./tools/start-testnet.sh

# 3. Deploy a contract template
./tools/deploy-contract.sh templates/token.zkasm

# 4. Run tests
./tools/test-contract.sh templates/token.zkasm
```

## 📦 What's Included

### Contract Templates

Located in `templates/`:

- **`token.zkasm`** - Fungible token implementation (ERC-20-like)
- **`nft.zkasm`** - Non-fungible token implementation (ERC-721-like)
- **`escrow.zkasm`** - Trustless escrow contract

### Development Tools

Located in `tools/`:

- **`start-testnet.sh`** - Launch a local BitCell testnet
- **`deploy-contract.sh`** - Deploy contracts to testnet/mainnet
- **`test-contract.sh`** - Run contract tests
- **`compile-contract.sh`** - Compile ZKASM to bytecode

### Documentation

Located in `docs/`:

- **`GETTING_STARTED.md`** - Step-by-step tutorial
- **`API_REFERENCE.md`** - Complete API documentation
- **`DEPLOYMENT_GUIDE.md`** - Production deployment guide
- **`BEST_PRACTICES.md`** - Security and optimization tips

### Examples

Located in `examples/`:

- Working examples demonstrating contract usage
- Integration patterns
- Advanced features

## 🔧 Requirements

- Rust 1.82+
- BitCell node (for testnet)
- 4GB+ RAM for local development

## 📚 Learning Path

1. Start with `docs/GETTING_STARTED.md`
2. Study the token template in `templates/token.zkasm`
3. Deploy to local testnet using `tools/`
4. Review `docs/API_REFERENCE.md` for details
5. Build your own contracts!

## 🛠️ ZKVM Instruction Set

BitCell smart contracts run on the ZKVM, a RISC-like VM with ZK-SNARK verification:

- 32 general-purpose registers (r0-r31)
- 1MB sparse memory address space
- Gas metering for resource control
- Field-friendly operations for efficient ZK proofs

See `docs/API_REFERENCE.md` for complete instruction set documentation.

## 🎯 Example: Token Transfer

```zkasm
# Load sender balance from state
LOAD r1, r0, sender_balance_addr

# Load transfer amount from input
LOAD r2, r0, amount_addr

# Check sufficient balance: sender_balance >= amount
LT r3, r1, r2              # r3 = (r1 < r2) ? 1 : 0
JZ r3, 0, sufficient       # Jump if sufficient balance

# Insufficient balance - revert
HALT

sufficient:
# Subtract from sender
SUB r1, r1, r2
STORE r0, r1, sender_balance_addr

# Load recipient balance
LOAD r4, r0, recipient_balance_addr

# Add to recipient
ADD r4, r4, r2
STORE r0, r4, recipient_balance_addr

# Success
HALT
```

## 🔐 Privacy Features

All BitCell contracts benefit from:

- **Zero-knowledge execution proofs** - Prove correct execution without revealing inputs
- **Private state** - Contract state hidden via Pedersen commitments
- **Gas multipliers** - 2x privacy bonus for private contracts

## 🤝 Contributing

Found a bug or want to add a template? Open an issue or PR in the main BitCell repository!

## 📄 License

MIT OR Apache-2.0 (same as BitCell core)
