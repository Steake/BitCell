//! Block API endpoints

use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use serde::Serialize;
use std::sync::Arc;

use crate::AppState;
use bitcell_ca::{Battle, BattleOutcome, Glider, GliderPattern, Position};

#[derive(Debug, Serialize)]
pub struct BlockInfo {
    pub height: u64,
    pub hash: String,
    pub timestamp: u64,
    pub proposer: String,
    pub transaction_count: usize,
    pub battle_count: usize,
}

#[derive(Debug, Serialize)]
pub struct BlockListResponse {
    pub blocks: Vec<BlockInfo>,
    pub total: usize,
}

#[derive(Debug, Serialize)]
pub struct BlockDetailResponse {
    pub height: u64,
    pub hash: String,
    pub timestamp: u64,
    pub proposer: String,
    pub prev_hash: String,
    pub tx_root: String,
    pub state_root: String,
    pub transactions: Vec<TransactionInfo>,
    pub battle_count: usize,
}

#[derive(Debug, Serialize)]
pub struct TransactionInfo {
    pub hash: String,
    pub from: String,
    pub to: String,
    pub amount: u64,
}

#[derive(Debug, Serialize)]
pub struct BlockBattleFrame {
    pub step: usize,
    pub grid: Vec<Vec<u8>>,
    pub energy_a: u64,
    pub energy_b: u64,
}

#[derive(Debug, Serialize)]
pub struct BlockBattleVisualization {
    pub block_height: u64,
    pub battle_index: usize,
    pub glider_a_pattern: String,
    pub glider_b_pattern: String,
    pub winner: String,
    pub steps: usize,
    pub frames: Vec<BlockBattleFrame>,
}

/// List recent blocks
pub async fn list_blocks(
    State(state): State<Arc<AppState>>,
) -> Result<Json<BlockListResponse>, (StatusCode, Json<String>)> {
    // Get all registered nodes
    let nodes = state.process.list_nodes();
    
    if nodes.is_empty() {
        return Err((
            StatusCode::SERVICE_UNAVAILABLE,
            Json("No nodes available. Please deploy nodes first.".to_string()),
        ));
    }

    // Try to fetch blocks from the first running node
    // In a real implementation, this would query the blockchain via RPC
    // For now, we'll return mock data based on chain height from metrics
    
    let endpoints: Vec<(String, String)> = nodes
        .iter()
        .map(|n| {
            let metrics_port = n.port + 1;
            (n.id.clone(), format!("http://127.0.0.1:{}/metrics", metrics_port))
        })
        .collect();

    if endpoints.is_empty() {
        return Err((
            StatusCode::SERVICE_UNAVAILABLE,
            Json("No running nodes found.".to_string()),
        ));
    }

    // Fetch current chain height
    let aggregated = state.metrics_client.aggregate_metrics(&endpoints)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(e)))?;

    let chain_height = aggregated.chain_height;
    
    // Generate mock block list (most recent 10 blocks)
    let start_height = chain_height.saturating_sub(9);
    let mut blocks = Vec::new();
    
    for height in start_height..=chain_height {
        blocks.push(BlockInfo {
            height,
            hash: format!("0x{:016x}", height * 12345),
            timestamp: 1700000000 + (height * 600), // 10 min blocks
            proposer: format!("miner-{}", height % 3),
            transaction_count: (height % 5) as usize,
            battle_count: 1, // Each block has 1 battle in simplified model
        });
    }
    
    // Reverse to show newest first
    blocks.reverse();

    Ok(Json(BlockListResponse {
        total: blocks.len(),
        blocks,
    }))
}

/// Get block details by height
pub async fn get_block(
    State(state): State<Arc<AppState>>,
    Path(height): Path<u64>,
) -> Result<Json<BlockDetailResponse>, (StatusCode, Json<String>)> {
    // In a real implementation, this would fetch the actual block from the blockchain
    // For now, return mock data
    
    let nodes = state.process.list_nodes();
    if nodes.is_empty() {
        return Err((
            StatusCode::SERVICE_UNAVAILABLE,
            Json("No nodes available.".to_string()),
        ));
    }

    // Handle edge case of height == 0 to prevent underflow
    if height == 0 {
        return Err((
            StatusCode::BAD_REQUEST,
            Json("Invalid block height: cannot be 0".to_string()),
        ));
    }

    Ok(Json(BlockDetailResponse {
        height,
        hash: format!("0x{:016x}", height * 12345),
        timestamp: 1700000000 + (height * 600),
        proposer: format!("miner-{}", height % 3),
        prev_hash: format!("0x{:016x}", (height - 1) * 12345),
        tx_root: format!("0x{:016x}", height * 54321),
        state_root: format!("0x{:016x}", height * 98765),
        transactions: vec![],
        battle_count: 1,
    }))
}

/// Get battle visualization for a specific block
pub async fn get_block_battles(
    State(_state): State<Arc<AppState>>,
    Path(height): Path<u64>,
) -> Result<Json<Vec<BlockBattleVisualization>>, (StatusCode, Json<String>)> {
    tracing::info!("Fetching battle visualization for block {}", height);

    // In a real implementation, we would:
    // 1. Fetch the block from the blockchain
    // 2. Extract the glider reveals from the tournament data
    // 3. Re-simulate the battles
    //
    // For now, we'll simulate a deterministic battle based on block height
    // to demonstrate the visualization
    
    let battle_index = 0;
    
    // Deterministically choose glider patterns based on block height
    let patterns = [
        GliderPattern::Standard,
        GliderPattern::Lightweight,
        GliderPattern::Middleweight,
        GliderPattern::Heavyweight,
    ];
    
    let pattern_a = patterns[(height % 4) as usize];
    let pattern_b = patterns[((height + 1) % 4) as usize];
    
    // Create gliders
    let glider_a = Glider::new(pattern_a, Position::new(256, 512));
    let glider_b = Glider::new(pattern_b, Position::new(768, 512));
    
    // Create battle with fewer steps for faster rendering
    let steps = 500;
    let frame_count = 20;
    let downsample_size = 128;
    
    // Generate entropy seed from block height
    let mut entropy_seed = [0u8; 32];
    let height_bytes = height.to_le_bytes();
    // Fill entropy seed with deterministic but varied data based on height
    for i in 0..32 {
        entropy_seed[i] = height_bytes[i % 8].wrapping_mul((i as u8).wrapping_add(1));
    }
    
    let battle = Battle::with_entropy(glider_a, glider_b, steps, entropy_seed);
    
    // Calculate sample steps
    let sample_interval = steps / frame_count;
    let mut sample_steps: Vec<usize> = (0..frame_count)
        .map(|i| i * sample_interval)
        .collect();
    sample_steps.push(steps);
    
    // Run simulation in blocking task
    let (outcome, frames) = tokio::task::spawn_blocking(move || {
        let outcome = battle.simulate();
        let grids = battle.grid_states(&sample_steps);
        
        let mut frames = Vec::new();
        for (i, grid) in grids.iter().enumerate() {
            let step = sample_steps[i];
            let (energy_a, energy_b) = battle.measure_regional_energy(grid);
            let downsampled = grid.downsample(downsample_size);
            
            frames.push(BlockBattleFrame {
                step,
                grid: downsampled,
                energy_a,
                energy_b,
            });
        }
        
        (outcome, frames)
    })
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(format!("Task join error: {}", e))))?;
    
    let winner = match outcome {
        BattleOutcome::AWins => "glider_a",
        BattleOutcome::BWins => "glider_b",
        BattleOutcome::Tie => "tie",
    };
    
    let pattern_name = |p: GliderPattern| match p {
        GliderPattern::Standard => "Standard",
        GliderPattern::Lightweight => "Lightweight",
        GliderPattern::Middleweight => "Middleweight",
        GliderPattern::Heavyweight => "Heavyweight",
    };
    
    let visualization = BlockBattleVisualization {
        block_height: height,
        battle_index,
        glider_a_pattern: pattern_name(pattern_a).to_string(),
        glider_b_pattern: pattern_name(pattern_b).to_string(),
        winner: winner.to_string(),
        steps,
        frames,
    };
    
    Ok(Json(vec![visualization]))
}
