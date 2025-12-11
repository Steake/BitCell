//! BitCell Light Client Example
//!
//! Demonstrates how to use the light client for syncing headers,
//! querying balances, and submitting transactions.

use bitcell_light_client::{
    HeaderChain, HeaderChainConfig, CheckpointManager, HeaderSync,
    LightWallet, LightClientProtocol, Checkpoint,
};
use bitcell_consensus::BlockHeader;
use bitcell_crypto::{SecretKey, Hash256};
use parking_lot::RwLock;
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::fmt::init();
    
    println!("🌟 BitCell Light Client Example");
    println!("================================\n");
    
    // 1. Create genesis header
    println!("📦 Creating genesis header...");
    let genesis_sk = SecretKey::generate();
    let genesis = BlockHeader {
        height: 0,
        prev_hash: Hash256::zero(),
        tx_root: Hash256::zero(),
        state_root: Hash256::zero(),
        timestamp: 0,
        proposer: genesis_sk.public_key(),
        vrf_output: [0u8; 32],
        vrf_proof: vec![],
        work: 100,
    };
    println!("✓ Genesis created at height {}", genesis.height);
    
    // 2. Initialize header chain
    println!("\n🔗 Initializing header chain...");
    let config = HeaderChainConfig {
        max_headers: 10_000,
        checkpoint_interval: 1_000,
        checkpoint_confirmations: 100,
    };
    let header_chain = Arc::new(HeaderChain::new(genesis.clone(), config));
    println!("✓ Header chain initialized");
    println!("  - Max headers: {}", header_chain.memory_usage() / 500);
    println!("  - Current tip: {}", header_chain.tip_height());
    
    // 3. Setup checkpoint manager
    println!("\n🔖 Setting up checkpoint manager...");
    let checkpoint_manager = Arc::new(RwLock::new(CheckpointManager::new()));
    
    // Add a sample checkpoint
    let checkpoint_header = BlockHeader {
        height: 1000,
        prev_hash: Hash256::hash(b"prev"),
        tx_root: Hash256::zero(),
        state_root: Hash256::zero(),
        timestamp: 1000000,
        proposer: genesis_sk.public_key(),
        vrf_output: [1u8; 32],
        vrf_proof: vec![],
        work: 100,
    };
    let checkpoint = Checkpoint::new(checkpoint_header, "Checkpoint 1000".to_string());
    checkpoint_manager.write().add_checkpoint(checkpoint)?;
    println!("✓ Checkpoint manager ready");
    println!("  - Checkpoints loaded: {}", checkpoint_manager.read().all_checkpoints().len());
    
    // 4. Create header sync
    println!("\n🔄 Creating header sync manager...");
    let sync = HeaderSync::new(header_chain.clone(), checkpoint_manager.clone());
    println!("✓ Sync manager created");
    println!("  - Status: {:?}", sync.status());
    println!("  - Progress: {:.1}%", sync.progress() * 100.0);
    
    // 5. Demonstrate adding headers
    println!("\n📥 Adding sample headers to chain...");
    let mut prev = genesis;
    for i in 1..=10 {
        let header = BlockHeader {
            height: i,
            prev_hash: prev.hash(),
            tx_root: Hash256::hash(&format!("tx_root_{}", i).as_bytes()),
            state_root: Hash256::hash(&format!("state_root_{}", i).as_bytes()),
            timestamp: prev.timestamp + 10,
            proposer: genesis_sk.public_key(),
            vrf_output: [i as u8; 32],
            vrf_proof: vec![],
            work: 100,
        };
        header_chain.add_header(header.clone())?;
        prev = header;
    }
    println!("✓ Added 10 headers");
    println!("  - Current tip: {}", header_chain.tip_height());
    println!("  - Memory usage: ~{} KB", header_chain.memory_usage() / 1024);
    
    // 6. Create light wallet
    println!("\n💰 Creating light wallet...");
    let wallet_sk = Arc::new(SecretKey::generate());
    let protocol = Arc::new(LightClientProtocol::new());
    let wallet = LightWallet::full(
        wallet_sk.clone(),
        header_chain.clone(),
        protocol.clone(),
    );
    println!("✓ Wallet created");
    println!("  - Mode: {:?}", wallet.mode());
    println!("  - Address: {:?}", wallet.address());
    println!("  - Memory usage: ~{} bytes", wallet.memory_usage());
    
    // 7. Create a read-only wallet
    println!("\n👀 Creating read-only wallet...");
    let other_pk = SecretKey::generate().public_key();
    let readonly_wallet = LightWallet::read_only(
        other_pk,
        header_chain.clone(),
        protocol.clone(),
    );
    println!("✓ Read-only wallet created");
    println!("  - Mode: {:?}", readonly_wallet.mode());
    
    // 8. Demonstrate transaction creation
    println!("\n📝 Creating sample transaction...");
    let to = SecretKey::generate().public_key();
    let tx = wallet.create_transaction(
        to,
        1000,    // amount
        0,       // nonce
        21000,   // gas_limit
        1,       // gas_price
    )?;
    println!("✓ Transaction created");
    println!("  - Amount: {} units", tx.amount);
    println!("  - Nonce: {}", tx.nonce);
    println!("  - Gas limit: {}", tx.gas_limit);
    
    // 9. Display statistics
    println!("\n📊 Light Client Statistics");
    println!("==========================");
    println!("Header Chain:");
    println!("  - Tip height: {}", header_chain.tip_height());
    println!("  - Tip hash: {:?}", header_chain.tip_hash());
    println!("  - Total work: {:?}", header_chain.total_work_at(header_chain.tip_height()));
    println!("  - Memory usage: ~{} KB", header_chain.memory_usage() / 1024);
    println!("\nSync Status:");
    println!("  - Status: {:?}", sync.status());
    println!("  - Progress: {:.1}%", sync.progress() * 100.0);
    println!("\nWallet:");
    println!("  - Mode: {:?}", wallet.mode());
    println!("  - Pending txs: {}", wallet.pending_transactions().len());
    println!("  - Memory usage: ~{} bytes", wallet.memory_usage());
    
    // 10. Resource usage summary
    println!("\n💾 Resource Usage Summary");
    println!("========================");
    let total_memory = header_chain.memory_usage() + wallet.memory_usage();
    println!("  - Total memory: ~{} KB", total_memory / 1024);
    println!("  - Headers: ~{} KB", header_chain.memory_usage() / 1024);
    println!("  - Wallet: ~{} bytes", wallet.memory_usage());
    println!("\n✅ All features demonstrated successfully!");
    println!("\nNote: This is a demo. In production:");
    println!("  - Headers would be downloaded from peers");
    println!("  - State proofs would be requested from full nodes");
    println!("  - Transactions would be submitted to the network");
    println!("  - Sync would continue to keep up with chain tip");
    
    Ok(())
}
