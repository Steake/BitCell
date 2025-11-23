//! Testing utilities API endpoints

use axum::{
    extract::State,
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::AppState;

// Import BitCell types
use bitcell_ca::{Battle, Glider, GliderPattern, Position, BattleOutcome};

#[derive(Debug, Deserialize)]
pub struct RunBattleTestRequest {
    pub glider_a: String,
    pub glider_b: String,
    pub steps: Option<usize>,
}

#[derive(Debug, Serialize)]
pub struct BattleTestResponse {
    pub test_id: String,
    pub winner: String,
    pub steps: usize,
    pub final_energy_a: u64,
    pub final_energy_b: u64,
    pub duration_ms: u64,
}

#[derive(Debug, Deserialize)]
pub struct BattleVisualizationRequest {
    pub glider_a: String,
    pub glider_b: String,
    pub steps: Option<usize>,
    pub frame_count: Option<usize>,
    pub downsample_size: Option<usize>,
}

#[derive(Debug, Serialize)]
pub struct BattleVisualizationResponse {
    pub test_id: String,
    pub winner: String,
    pub steps: usize,
    pub final_energy_a: u64,
    pub final_energy_b: u64,
    pub frames: Vec<BattleFrame>,
}

#[derive(Debug, Serialize)]
pub struct BattleFrame {
    pub step: usize,
    pub grid: Vec<Vec<u8>>,
    pub energy_a: u64,
    pub energy_b: u64,
}

#[derive(Debug, Deserialize)]
pub struct SendTestTransactionRequest {
    pub from: Option<String>,
    pub to: String,
    pub amount: u64,
}

#[derive(Debug, Serialize)]
pub struct TransactionTestResponse {
    pub tx_hash: String,
    pub status: String,
    pub message: String,
}

fn parse_glider_pattern(name: &str) -> Result<GliderPattern, String> {
    match name.to_lowercase().as_str() {
        "standard" => Ok(GliderPattern::Standard),
        "lightweight" | "lwss" => Ok(GliderPattern::Lightweight),
        "middleweight" | "mwss" => Ok(GliderPattern::Middleweight),
        "heavyweight" | "hwss" => Ok(GliderPattern::Heavyweight),
        _ => Err(format!("Unknown glider pattern: {}", name)),
    }
}

/// Run a battle test
pub async fn run_battle_test(
    State(_state): State<Arc<AppState>>,
    Json(req): Json<RunBattleTestRequest>,
) -> Result<Json<BattleTestResponse>, (StatusCode, Json<String>)> {
    let test_id = format!("test-{}", chrono::Utc::now().timestamp());

    tracing::info!("Running battle test: {} vs {}", req.glider_a, req.glider_b);

    // Parse glider patterns
    let pattern_a = parse_glider_pattern(&req.glider_a)
        .map_err(|e| (StatusCode::BAD_REQUEST, Json(e)))?;

    let pattern_b = parse_glider_pattern(&req.glider_b)
        .map_err(|e| (StatusCode::BAD_REQUEST, Json(e)))?;

    // Create gliders
    let glider_a = Glider::new(pattern_a, Position::new(256, 512));
    let glider_b = Glider::new(pattern_b, Position::new(768, 512));

    // Create battle
    let steps = req.steps.unwrap_or(1000);
    let battle = if steps != 1000 {
        Battle::with_steps(glider_a, glider_b, steps)
    } else {
        Battle::new(glider_a, glider_b)
    };

    // Run battle simulation
    let start = std::time::Instant::now();

    let (outcome, energy_a, energy_b) = tokio::task::spawn_blocking(move || {
        // Simulate the battle
        let outcome = battle.simulate()
            .map_err(|e| format!("Battle simulation error: {:?}", e))?;

        // Get final grid to measure energies
        let final_grid = battle.final_grid();
        let (energy_a, energy_b) = battle.measure_regional_energy(&final_grid);

        Ok::<_, String>((outcome, energy_a, energy_b))
    })
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(format!("Task join error: {}", e))))?
    .map_err(|e: String| (StatusCode::INTERNAL_SERVER_ERROR, Json(e)))?;

    let duration = start.elapsed();

    let winner = match outcome {
        BattleOutcome::AWins => "glider_a".to_string(),
        BattleOutcome::BWins => "glider_b".to_string(),
        BattleOutcome::Tie => "tie".to_string(),
    };

    tracing::info!(
        "Battle test completed: winner={}, energy_a={}, energy_b={}, duration={}ms",
        winner,
        energy_a,
        energy_b,
        duration.as_millis()
    );

    let response = BattleTestResponse {
        test_id,
        winner,
        steps,
        final_energy_a: energy_a,
        final_energy_b: energy_b,
        duration_ms: duration.as_millis() as u64,
    };

    Ok(Json(response))
}

/// Send a test transaction
pub async fn send_test_transaction(
    State(_state): State<Arc<AppState>>,
    Json(req): Json<SendTestTransactionRequest>,
) -> Result<Json<TransactionTestResponse>, (StatusCode, Json<String>)> {
    // TODO: Actually send transaction to a running node
    // For now, return a formatted response

    let tx_hash = format!("0x{:x}", chrono::Utc::now().timestamp());

    let response = TransactionTestResponse {
        tx_hash,
        status: "pending".to_string(),
        message: format!(
            "Test transaction sent: {} -> {} ({} units)",
            req.from.unwrap_or_else(|| "genesis".to_string()),
            req.to,
            req.amount
        ),
    };

    tracing::info!("Test transaction: {}", response.message);

    Ok(Json(response))
}

/// Run a battle with visualization frames
pub async fn run_battle_visualization(
    State(_state): State<Arc<AppState>>,
    Json(req): Json<BattleVisualizationRequest>,
) -> Result<Json<BattleVisualizationResponse>, (StatusCode, Json<String>)> {
    let test_id = format!("viz-{}", chrono::Utc::now().timestamp());

    tracing::info!("Running battle visualization: {} vs {}", req.glider_a, req.glider_b);

    // Parse glider patterns
    let pattern_a = parse_glider_pattern(&req.glider_a)
        .map_err(|e| (StatusCode::BAD_REQUEST, Json(e)))?;

    let pattern_b = parse_glider_pattern(&req.glider_b)
        .map_err(|e| (StatusCode::BAD_REQUEST, Json(e)))?;

    // Create gliders
    let glider_a = Glider::new(pattern_a, Position::new(256, 512));
    let glider_b = Glider::new(pattern_b, Position::new(768, 512));

    // Create battle
    let steps = req.steps.unwrap_or(1000);
    let frame_count = req.frame_count.unwrap_or(20).min(100); // Max 100 frames
    let downsample_size = req.downsample_size.unwrap_or(128).min(512); // Max 512x512

    let battle = if steps != 1000 {
        Battle::with_steps(glider_a, glider_b, steps)
    } else {
        Battle::new(glider_a, glider_b)
    };

    // Calculate which steps to capture
    let sample_interval = steps / frame_count;
    let mut sample_steps: Vec<usize> = (0..frame_count)
        .map(|i| i * sample_interval)
        .collect();
    sample_steps.push(steps); // Always include final step

    // Run simulation and capture frames
    let (outcome, frames) = tokio::task::spawn_blocking(move || {
        // Get outcome
        let outcome = battle.simulate()
            .map_err(|e| format!("Battle simulation error: {:?}", e))?;

        // Get grid states at sample steps
        let grids = battle.grid_states(&sample_steps);

        // Create frames with downsampled grids and energy measurements
        let mut frames = Vec::new();
        for (i, grid) in grids.iter().enumerate() {
            let step = sample_steps[i];
            let (energy_a, energy_b) = battle.measure_regional_energy(grid);
            let downsampled = grid.downsample(downsample_size);

            frames.push(BattleFrame {
                step,
                grid: downsampled,
                energy_a,
                energy_b,
            });
        }

        Ok::<_, String>((outcome, frames))
    })
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(format!("Task join error: {}", e))))?
    .map_err(|e: String| (StatusCode::INTERNAL_SERVER_ERROR, Json(e)))?;

    let winner = match outcome {
        BattleOutcome::AWins => "glider_a".to_string(),
        BattleOutcome::BWins => "glider_b".to_string(),
        BattleOutcome::Tie => "tie".to_string(),
    };

    let final_energy_a = frames.last().map(|f| f.energy_a).unwrap_or(0);
    let final_energy_b = frames.last().map(|f| f.energy_b).unwrap_or(0);

    tracing::info!(
        "Battle visualization completed: winner={}, {} frames captured",
        winner,
        frames.len()
    );

    let response = BattleVisualizationResponse {
        test_id,
        winner,
        steps,
        final_energy_a,
        final_energy_b,
        frames,
    };

    Ok(Json(response))
}
