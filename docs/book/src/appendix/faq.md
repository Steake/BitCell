# Frequently Asked Questions

Common questions about BitCell and their answers.

## General

### What is BitCell?

BitCell is a blockchain protocol that uses Conway's Game of Life tournaments for consensus instead of traditional proof-of-work mining. Miners compete by designing glider patterns that battle in a cellular automaton arena.

### Is this a joke?

No! While the concept is unconventional, BitCell is a serious blockchain protocol with:
- Deterministic consensus (CA battles have predictable outcomes)
- Zero-knowledge privacy (Groth16 SNARKs)
- Anti-cartel design (random pairings, ring signatures)
- Real economic incentives (block rewards, fees)

### Why not just use Bitcoin's PoW or Ethereum's PoS?

Bitcoin's PoW:
- ❌ Energy-intensive hash grinding
- ❌ Mining centralization (ASIC farms)
- ❌ No inherent value in work

Ethereum's PoS:
- ❌ Plutocratic (rich get richer)
- ❌ Capital requirements exclude many
- ❌ Less decentralized

BitCell's Tournament:
- ✅ Interesting work (pattern design)
- ✅ Skill-based competition
- ✅ Random pairings prevent cartels
- ✅ Lower energy usage

## Technical

### What's the TPS (transactions per second)?

~100 TPS. We prioritize security and decentralization over raw throughput.

For comparison:
- Bitcoin: ~7 TPS
- Ethereum: ~15 TPS
- BitCell: ~100 TPS

### Is it quantum-resistant?

Partially. The CA battle mechanism is fundamentally quantum-resistant (deterministic classical computation). However, current signatures (ECDSA) are vulnerable to quantum attacks. We plan to upgrade to post-quantum signatures (e.g., SPHINCS+) before quantum computers become a threat.

### How big is the blockchain?

Current testnet:
- Block size: ~500KB average
- Daily growth: ~40GB per day
- Annual growth: ~15TB per year

Archive node: Stores full history  
Pruned node: Stores recent state (~100GB)  
Light client: Headers only (~10MB)

### Can I run a node on a Raspberry Pi?

Validator: Probably not (ZK proving is CPU/memory intensive)  
Light client: Yes! Light clients only verify headers and can run on low-power devices.

## Mining & Tournaments

### How do I mine BitCell?

See [Miner Node Setup](../node/miner-setup.md) for complete instructions.

Quick version:
1. Run a full node
2. Build trust score to 0.75+ (take ~1 week of honest participation)
3. Lock minimum bond (1000 CELL)
4. Design glider patterns
5. Participate in tournaments

### Can I win by just using the biggest glider?

Initially, yes. But as the network matures:
- Larger patterns are more predictable
- Lightweight patterns can outmaneuver heavy ones
- Pattern diversity wins tournaments
- Strategy matters more than size

### What's a good glider pattern?

It depends on your strategy:
- **Aggressive**: Heavyweight Spaceship (HWSS) - high energy, slow
- **Balanced**: Middleweight Spaceship (MWSS) - medium energy, medium speed
- **Evasive**: Lightweight Spaceship (LWSS) - low energy, fast
- **Standard**: Glider - classic choice

See [Glider Patterns](../concepts/glider-patterns.md) for details.

### How long do battles take?

1000 steps in the CA simulation:
- CPU: ~5-10 seconds (1024×1024 grid)
- GPU: ~0.5-1 seconds (with CUDA/OpenCL)

Proof generation:
- ~10-30 seconds (Groth16 proof)

Total per battle: ~15-40 seconds

### Can I mine on a laptop?

Technically yes, but not recommended:
- Battles are CPU-intensive
- Proof generation requires significant RAM
- You'll compete against optimized mining rigs

Better options:
- Cloud computing (AWS, GCP)
- Dedicated mining hardware
- Mining pools (coming in RC3)

## Economics

### What's the total supply?

~21 million CELL (similar to Bitcoin)

Block reward starts at 50 CELL and halves every 210,000 blocks (~4 years).

### How are rewards distributed?

Per block:
- 60% → Tournament winner (proposer)
- 30% → All participants (weighted by round reached)
- 10% → Treasury/dev fund

Example: 50 CELL reward
- Winner: 30 CELL
- Participants: 15 CELL (split among all)
- Treasury: 5 CELL

### What can I do with CELL tokens?

- Pay transaction fees
- Deploy smart contracts
- Lock as miner bond
- Participate in governance (future)
- Trade on exchanges (future)

### Where can I buy CELL?

Currently:
- Mine them (participate in tournaments)
- Testnet faucet (for testing)

Future:
- Exchanges (post-mainnet launch)
- OTC markets
- DEXs (decentralized exchanges)

## Wallets

### Which wallet should I use?

For beginners: GUI Wallet
- User-friendly interface
- Tournament visualization
- Built-in explorer

For advanced users: CLI Wallet
- Full control
- Scriptable
- Lower resource usage

For large amounts: Hardware wallet (future RC2)
- Ledger/Trezor support
- Maximum security

### How do I backup my wallet?

**Recovery phrase** (most important):
```bash
# Show recovery phrase
bitcell-wallet show-phrase

# Write down all 12/24 words in order
# Store in secure location (not digitally!)
```

**Private key** (alternative):
```bash
# Export private key
bitcell-wallet export-key
# Save securely, never share
```

**Wallet file** (convenient):
```bash
# Backup wallet file
cp ~/.bitcell/wallet.dat /secure/backup/
```

### I lost my recovery phrase. Can I recover my wallet?

No. If you lose your recovery phrase and don't have another backup (private key or wallet file), your funds are **permanently lost**. There is no customer service or password reset.

This is a fundamental property of blockchain: You control your keys, you control your funds. Nobody else can help.

### How do I keep my wallet secure?

Security checklist:
- ✅ Write down recovery phrase offline
- ✅ Store in multiple secure locations
- ✅ Never share private keys
- ✅ Use strong wallet password
- ✅ Keep software updated
- ✅ Verify addresses before sending
- ✅ Start with small test transactions
- ❌ Never store keys digitally (cloud, email, etc.)
- ❌ Never enter keys on websites
- ❌ Never share screen/keys during support

## Smart Contracts

### What languages can I use?

Two options:

**BCL** (BitCell Language) - High-level, Solidity-like:
```bcl
contract Token {
    mapping(address => uint) balances;
    
    function transfer(address to, uint amount) {
        require(balances[msg.sender] >= amount);
        balances[msg.sender] -= amount;
        balances[to] += amount;
    }
}
```

**ZKASM** - Assembly for full control:
```zkasm
FUNCTION transfer:
    LOAD r1, sender
    LOAD r2, amount
    CALL _do_transfer
    RET
```

See [Smart Contracts](../contracts/zkvm-overview.md) for details.

### Are contracts private?

Yes! All contract state is encrypted using Pedersen commitments. Execution happens off-chain with zero-knowledge proofs. Validators verify correctness without seeing plaintext data.

### Can contracts call other contracts?

Yes, contracts are fully composable. You can call other contracts while maintaining privacy for both.

### What's the gas limit?

Per transaction: 10,000,000 gas  
Per block: 100,000,000 gas

Typical costs:
- Simple transfer: 21,000 gas
- Token transfer: ~50,000 gas
- Contract deployment: 500,000+ gas

## Network

### What network should I use?

**Testnet** (recommended for learning):
- Free tokens from faucet
- Experiment safely
- Reset periodically

**Mainnet** (future):
- Real value
- Permanent state
- Production use

### How do I connect to testnet?

```bash
bitcell-node init --network testnet
bitcell-node start --validator
```

### How many peers should I have?

Healthy node: 8-50 peers  
Excellent node: 50-100 peers  
Too few: < 8 peers (check firewall)  
Too many: > 200 peers (may indicate issue)

### My node won't sync. What do I do?

Troubleshooting:
1. Check internet connection
2. Open firewall ports (30303 TCP/UDP)
3. Check peer count: `bitcell-node peers`
4. Try different bootstrap nodes
5. Check logs: `~/.bitcell/logs/node.log`

See [Troubleshooting](./troubleshooting.md) for more.

## Development

### How do I contribute?

See [Contributing Guide](../development/contributing.md).

Ways to contribute:
- Code contributions (Rust)
- Documentation improvements
- Bug reports
- Feature suggestions
- Testing and QA
- Community support

### Where's the source code?

GitHub: https://github.com/Steake/BitCell

License: Dual MIT / Apache 2.0

### Can I build a business on BitCell?

Yes! The protocol is open source and permissionless. You can:
- Build dApps
- Run infrastructure (nodes, explorers)
- Create developer tools
- Offer services

## Community

### Where can I get help?

- GitHub Issues: https://github.com/Steake/BitCell/issues
- Documentation: https://docs.bitcell.network
- Discord: Coming soon
- Forum: Coming soon

### How do I stay updated?

- Twitter: @bitcell_net (coming soon)
- Blog: blog.bitcell.network (coming soon)
- Release notes: See [Changelog](./changelog.md)

### Is there a bug bounty?

Not yet, but planned for mainnet launch. Current status is testnet/RC, so we encourage responsible disclosure of any security issues to security@bitcell.network.

## Troubleshooting

### "Trust score below threshold"

New nodes start with trust 0.40, below eligibility threshold 0.75. Build reputation by:
- Running a validator consistently
- Submitting valid blocks
- Participating honestly

Takes ~1 week of consistent operation.

### "Insufficient funds for gas"

You need CELL tokens to pay transaction fees. Get testnet tokens:
```bash
curl -X POST https://faucet.testnet.bitcell.network/request \
  -d '{"address": "YOUR_ADDRESS"}'
```

### "Transaction reverted"

Your transaction was rejected. Common causes:
- Insufficient balance
- Invalid recipient address
- Contract execution failed
- Gas limit too low

Check transaction receipt for specific error.

### "Cannot connect to RPC"

Node isn't running or RPC is disabled:
```bash
# Check if node is running
bitcell-node status

# Start with RPC enabled
bitcell-node start --rpc --rpc-addr 0.0.0.0:8545
```

For more help, see [Troubleshooting Guide](./troubleshooting.md).
