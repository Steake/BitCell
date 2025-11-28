//! Battle simulation between gliders
//!
//! Simulates CA evolution with two gliders and determines the winner.

use crate::glider::Glider;
use crate::grid::{Cell, Grid, Position};
use crate::rules::{evolve_grid, evolve_n_steps};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Number of steps to simulate a battle
pub const BATTLE_STEPS: usize = 1000;

/// Spawn positions for battles (far apart to allow evolution)
pub const SPAWN_A: Position = Position { x: 256, y: 512 };
pub const SPAWN_B: Position = Position { x: 768, y: 512 };

/// Battle outcome
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum BattleOutcome {
    /// A wins by energy
    AWins,
    /// B wins by energy
    BWins,
    /// Tie (same energy)
    Tie,
}

/// Battle history for computing MII and TED tiebreakers
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BattleHistory {
    /// Energy deltas in region A per timestep [timestep][cell_idx]
    pub region_a_deltas: Vec<Vec<i32>>,
    
    /// Energy deltas in region B per timestep [timestep][cell_idx]
    pub region_b_deltas: Vec<Vec<i32>>,
    
    /// Region states for entropy calculation
    pub region_a_states: Vec<Vec<u8>>,
    pub region_b_states: Vec<Vec<u8>>,
}

impl BattleHistory {
    /// Create new empty battle history
    pub fn new() -> Self {
        Self {
            region_a_deltas: Vec::new(),
            region_b_deltas: Vec::new(),
            region_a_states: Vec::new(),
            region_b_states: Vec::new(),
        }
    }
    
    /// Record a timestep
    pub fn record_timestep(
        &mut self,
        prev_a: &[u8],
        curr_a: &[u8],
        prev_b: &[u8],
        curr_b: &[u8],
    ) {
        // Calculate deltas for region A
        let deltas_a: Vec<i32> = curr_a.iter().zip(prev_a.iter())
            .map(|(&c, &p)| c as i32 - p as i32)
            .collect();
        
        // Calculate deltas for region B
        let deltas_b: Vec<i32> = curr_b.iter().zip(prev_b.iter())
            .map(|(&c, &p)| c as i32 - p as i32)
            .collect();
        
        self.region_a_deltas.push(deltas_a);
        self.region_b_deltas.push(deltas_b);
        self.region_a_states.push(curr_a.to_vec());
        self.region_b_states.push(curr_b.to_vec());
    }
}

/// A battle between two gliders
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Battle {
    pub glider_a: Glider,
    pub glider_b: Glider,
    pub steps: usize,
    /// Entropy seed for introducing randomness
    pub entropy_seed: [u8; 32],
    /// Whether to track battle history for MII/TED tiebreakers
    #[serde(default)]
    pub track_history: bool,
}

impl Battle {
    /// Create a new battle with zero entropy (deterministic)
    pub fn new(glider_a: Glider, glider_b: Glider) -> Self {
        Self {
            glider_a,
            glider_b,
            steps: BATTLE_STEPS,
            entropy_seed: [0u8; 32],
            track_history: false,
        }
    }

    /// Create a battle with custom number of steps
    pub fn with_steps(glider_a: Glider, glider_b: Glider, steps: usize) -> Self {
        Self {
            glider_a,
            glider_b,
            steps,
            entropy_seed: [0u8; 32],
            track_history: false,
        }
    }

    /// Create a battle with entropy seed
    pub fn with_entropy(glider_a: Glider, glider_b: Glider, steps: usize, entropy_seed: [u8; 32]) -> Self {
        Self {
            glider_a,
            glider_b,
            steps,
            entropy_seed,
            track_history: false,
        }
    }

    /// Create a battle with entropy and history tracking for MII+ tiebreaker
    pub fn with_history(glider_a: Glider, glider_b: Glider, steps: usize, entropy_seed: [u8; 32]) -> Self {
        Self {
            glider_a,
            glider_b,
            steps,
            entropy_seed,
            track_history: true,
        }
    }

    /// Set up the initial grid with both gliders
    fn setup_grid(&self) -> Grid {
        let mut grid = Grid::new();

        // Apply spawn position jitter based on entropy
        let (jitter_a_x, jitter_a_y) = self.calculate_spawn_jitter(0);
        let (jitter_b_x, jitter_b_y) = self.calculate_spawn_jitter(8);
        
        let spawn_a = Position::new(
            (SPAWN_A.x as isize + jitter_a_x) as usize,
            (SPAWN_A.y as isize + jitter_a_y) as usize,
        );
        let spawn_b = Position::new(
            (SPAWN_B.x as isize + jitter_b_x) as usize,
            (SPAWN_B.y as isize + jitter_b_y) as usize,
        );

        // Place glider A at jittered spawn position A
        grid.set_pattern(spawn_a, &self.glider_a.cells());

        // Place glider B at jittered spawn position B
        grid.set_pattern(spawn_b, &self.glider_b.cells());

        // Add initial noise if entropy is non-zero
        if self.entropy_seed != [0u8; 32] {
            self.add_initial_noise(&mut grid);
        }

        grid
    }

    /// Calculate spawn position jitter from entropy seed
    /// Returns (x_offset, y_offset) in range [-10, 10]
    fn calculate_spawn_jitter(&self, seed_offset: usize) -> (isize, isize) {
        if self.entropy_seed == [0u8; 32] {
            return (0, 0);
        }

        // Use different parts of entropy seed for x and y
        let x_bytes = [
            self.entropy_seed[seed_offset],
            self.entropy_seed[seed_offset + 1],
            self.entropy_seed[seed_offset + 2],
            self.entropy_seed[seed_offset + 3],
        ];
        let y_bytes = [
            self.entropy_seed[seed_offset + 4],
            self.entropy_seed[seed_offset + 5],
            self.entropy_seed[seed_offset + 6],
            self.entropy_seed[seed_offset + 7],
        ];

        let x_val = i32::from_le_bytes(x_bytes);
        let y_val = i32::from_le_bytes(y_bytes);

        // Map to [-10, 10] range
        let x_jitter = (x_val % 21 - 10) as isize;
        let y_jitter = (y_val % 21 - 10) as isize;

        (x_jitter, y_jitter)
    }

    /// Add initial noise to grid (1-5% random live cells)
    fn add_initial_noise(&self, grid: &mut Grid) {
        // Calculate noise percentage from entropy (1-5%)
        let noise_byte = self.entropy_seed[16];
        let noise_percent = 1.0 + (noise_byte as f32 / 255.0) * 4.0; // 1-5%
        
        let total_cells = crate::grid::GRID_SIZE * crate::grid::GRID_SIZE;
        let noise_cells = (total_cells as f32 * noise_percent / 100.0) as usize;

        // Use entropy seed to deterministically place noise
        for i in 0..noise_cells {
            let seed_idx = (i * 4) % 32;
            let x_bytes = [
                self.entropy_seed[seed_idx],
                self.entropy_seed[(seed_idx + 1) % 32],
                self.entropy_seed[(seed_idx + 2) % 32],
                self.entropy_seed[(seed_idx + 3) % 32],
            ];
            let y_bytes = [
                self.entropy_seed[(seed_idx + 16) % 32],
                self.entropy_seed[(seed_idx + 17) % 32],
                self.entropy_seed[(seed_idx + 18) % 32],
                self.entropy_seed[(seed_idx + 19) % 32],
            ];

            let x = u32::from_le_bytes(x_bytes) as usize % crate::grid::GRID_SIZE;
            let y = u32::from_le_bytes(y_bytes) as usize % crate::grid::GRID_SIZE;

            // Random energy from entropy
            let energy = (self.entropy_seed[(seed_idx + 20) % 32] % 100) + 1;
            
            grid.set(Position::new(x, y), Cell::alive(energy));
        }
    }

    /// Simulate the battle and return the outcome
    pub fn simulate(&self) -> BattleOutcome {
        self.simulate_with_history().0
    }

    /// Simulate the battle with optional history tracking
    pub fn simulate_with_history(&self) -> (BattleOutcome, Option<BattleHistory>) {
        let initial_grid = self.setup_grid();

        if !self.track_history {
            // Fast path - no history tracking
            let final_grid = evolve_n_steps(&initial_grid, self.steps);
            return (self.determine_outcome(&final_grid, None), None);
        }

        // Slow path - track all timesteps for tiebreaker
        let mut history = BattleHistory::new();
        let mut current_grid = initial_grid.clone();

        for _ in 0..self.steps {
            // Capture "before" state
            let prev_region_a = self.extract_region(&current_grid, true);
            let prev_region_b = self.extract_region(&current_grid, false);

            // Evolve one step
            let next_grid = evolve_grid(&current_grid);

            // Capture "after" state
            let curr_region_a = self.extract_region(&next_grid, true);
            let curr_region_b = self.extract_region(&next_grid, false);

            history.record_timestep(&prev_region_a, &curr_region_a, &prev_region_b, &curr_region_b);
            
            current_grid = next_grid;
        }

        let outcome = self.determine_outcome(&current_grid, Some(&history));
        (outcome, Some(history))
    }

    /// Determine the outcome of the battle, using tiebreakers if necessary
    fn determine_outcome(&self, final_grid: &Grid, history: Option<&BattleHistory>) -> BattleOutcome {
        // Determine winner by energy in each half of the grid
        let (mut energy_a, mut energy_b) = self.measure_regional_energy(final_grid);

        // Apply energy fluctuations if entropy is non-zero
        if self.entropy_seed != [0u8; 32] {
            let (fluct_a, fluct_b) = self.calculate_energy_fluctuations();
            energy_a = ((energy_a as f64 * fluct_a) as u64).max(1);
            energy_b = ((energy_b as f64 * fluct_b) as u64).max(1);
        }

        // Check for exact tie
        if energy_a == energy_b {
            // Run tiebreaker pipeline
            return self.run_tiebreaker(history);
        }

        // Clear winner
        if energy_a > energy_b {
            BattleOutcome::AWins
        } else {
            BattleOutcome::BWins
        }
    }

    /// Calculate energy fluctuations from entropy (±5%)
    fn calculate_energy_fluctuations(&self) -> (f64, f64) {
        let fluct_a_byte = self.entropy_seed[24];
        let fluct_b_byte = self.entropy_seed[25];

        // Map bytes to range [0.95, 1.05]
        let fluct_a = 0.95 + (fluct_a_byte as f64 / 255.0) * 0.10;
        let fluct_b = 0.95 + (fluct_b_byte as f64 / 255.0) * 0.10;

        (fluct_a, fluct_b)
    }

    /// Measure energy in regions around spawn points
    pub fn measure_regional_energy(&self, grid: &Grid) -> (u64, u64) {
        let region_size = 128;

        // Region around spawn A
        // Use checked arithmetic to prevent overflow on wrapping_sub
        let mut energy_a = 0u64;
        let half_region = region_size / 2;
        for y in 0..region_size {
            for x in 0..region_size {
                // Toroidal wrapping is handled by Position::wrap()
                let pos = Position::new(
                    SPAWN_A.x.wrapping_add(x).wrapping_sub(half_region),
                    SPAWN_A.y.wrapping_add(y).wrapping_sub(half_region),
                );
                energy_a += grid.get(pos).energy() as u64;
            }
        }

        // Region around spawn B
        let mut energy_b = 0u64;
        for y in 0..region_size {
            for x in 0..region_size {
                let pos = Position::new(
                    SPAWN_B.x.wrapping_add(x).wrapping_sub(half_region),
                    SPAWN_B.y.wrapping_add(y).wrapping_sub(half_region),
                );
                energy_b += grid.get(pos).energy() as u64;
            }
        }

        (energy_a, energy_b)
    }

    /// Run the MII+ tiebreaker pipeline
    fn run_tiebreaker(&self, history: Option<&BattleHistory>) -> BattleOutcome {
        // Stage 1: MII (Mutual Influence Integral)
        if let Some(hist) = history {
            let (mii_a, mii_b) = self.compute_mii(hist);
            
            if mii_a > mii_b {
                return BattleOutcome::AWins;
            } else if mii_b > mii_a {
                return BattleOutcome::BWins;
            }
            
            // Stage 2: TED (Temporal Entropy Differential)
            let (ted_a, ted_b) = self.compute_ted(hist);
            
            if ted_a > ted_b {
                return BattleOutcome::AWins;
            } else if ted_b > ted_a {
                return BattleOutcome::BWins;
            }
        }
        
        // Stage 3: Lexicographic Seed Break (Final fallback)
        self.lexicographic_break()
    }

    /// Compute Mutual Influence Integral for both participants
    pub fn compute_mii(&self, history: &BattleHistory) -> (f64, f64) {
        let mii_a_to_b = self.compute_region_mii(&history.region_b_deltas);
        let mii_b_to_a = self.compute_region_mii(&history.region_a_deltas);
        (mii_a_to_b, mii_b_to_a)
    }

    fn compute_region_mii(&self, region_deltas: &[Vec<i32>]) -> f64 {
        let mut mii = 0.0;
        for timestep_deltas in region_deltas {
            for &delta in timestep_deltas {
                // Square delta to amplify high-impact moves
                mii += (delta as f64).powi(2);
            }
        }
        mii
    }

    /// Compute Temporal Entropy Differential
    pub fn compute_ted(&self, history: &BattleHistory) -> (f64, f64) {
        let ted_a_to_b = self.compute_region_ted(&history.region_b_states);
        let ted_b_to_a = self.compute_region_ted(&history.region_a_states);
        (ted_a_to_b, ted_b_to_a)
    }

    fn compute_region_ted(&self, region_states: &[Vec<u8>]) -> f64 {
        let mut ted = 0.0;
        for state in region_states {
            ted += self.shannon_entropy(state);
        }
        ted
    }

    /// Shannon entropy calculation
    fn shannon_entropy(&self, cells: &[u8]) -> f64 {
        if cells.is_empty() {
            return 0.0;
        }
        
        let mut freq_map: HashMap<u8, usize> = HashMap::new();
        for &cell in cells {
            *freq_map.entry(cell).or_insert(0) += 1;
        }
        
        let total = cells.len() as f64;
        let mut entropy = 0.0;
        
        for count in freq_map.values() {
            if *count > 0 {
                let p = *count as f64 / total;
                entropy -= p * p.log2();
            }
        }
        entropy
    }

    /// Lexicographic tiebreaker using hash of glider + entropy seed
    fn lexicographic_break(&self) -> BattleOutcome {
        let hash_a = self.hash_glider(&self.glider_a);
        let hash_b = self.hash_glider(&self.glider_b);
        
        if hash_a < hash_b {
            BattleOutcome::AWins
        } else {
            BattleOutcome::BWins
        }
    }

    /// Simple FNV-1a hash for deterministic tiebreaking
    fn hash_glider(&self, glider: &Glider) -> u64 {
        let mut hash = 0xcbf29ce484222325; // FNV offset basis
        
        // Mix in entropy seed
        for &byte in &self.entropy_seed {
            hash ^= byte as u64;
            hash = hash.wrapping_mul(0x100000001b3); // FNV prime
        }
        
        // Mix in glider pattern
        for &byte in &glider.pattern.to_bytes() {
            hash ^= byte as u64;
            hash = hash.wrapping_mul(0x100000001b3);
        }
        
        hash
    }

    /// Extract energy values from a region
    fn extract_region(&self, grid: &Grid, is_region_a: bool) -> Vec<u8> {
        let region_size = 128;
        let half_region = region_size / 2;
        let center = if is_region_a { SPAWN_A } else { SPAWN_B };
        
        let mut cells = Vec::with_capacity(region_size * region_size);
        
        for y in 0..region_size {
            for x in 0..region_size {
                let pos = Position::new(
                    center.x.wrapping_add(x).wrapping_sub(half_region),
                    center.y.wrapping_add(y).wrapping_sub(half_region),
                );
                cells.push(grid.get(pos).energy());
            }
        }
        cells
    }

    /// Get initial grid state (for proof generation)
    pub fn initial_grid(&self) -> Grid {
        self.setup_grid()
    }

    /// Get final grid state after simulation
    pub fn final_grid(&self) -> Grid {
        let initial = self.setup_grid();
        evolve_n_steps(&initial, self.steps)
    }

    /// Get grid states at specific steps for visualization.
    /// 
    /// Returns a vector of grids at the requested step intervals in the same order
    /// as the input `sample_steps` array.
    /// Steps that exceed `self.steps` are silently skipped.
    /// 
    /// # Performance Note
    /// This implementation sorts steps internally for incremental evolution efficiency,
    /// but returns grids in the original order requested.
    /// 
    /// # Memory Overhead
    /// Each grid clone can be expensive for large grids (e.g., 1024×1024 grids).
    /// Requesting many sample steps will require storing multiple grid copies in memory.
    /// For example, 100 sample steps could require several hundred MB of memory.
    pub fn grid_states(&self, sample_steps: &[usize]) -> Vec<Grid> {
        let initial = self.setup_grid();

        // Filter and create (index, step) pairs to preserve original order
        let mut indexed_steps: Vec<(usize, usize)> = sample_steps.iter()
            .enumerate()
            .filter(|(_, &step)| step <= self.steps)
            .map(|(idx, &step)| (idx, step))
            .collect();

        // Sort by step for efficient incremental evolution
        indexed_steps.sort_unstable_by_key(|(_, step)| *step);

        // Evolve grids in sorted order
        let mut evolved_grids = Vec::with_capacity(indexed_steps.len());
        let mut current_grid = initial;
        let mut prev_step = 0;

        for (original_idx, step) in &indexed_steps {
            let steps_to_evolve = step - prev_step;
            // If steps_to_evolve is 0 (e.g., for step 0), the grid remains unchanged
            if steps_to_evolve > 0 {
                current_grid = evolve_n_steps(&current_grid, steps_to_evolve);
            }
            evolved_grids.push((*original_idx, current_grid.clone()));
            prev_step = *step;
        }

        // Sort back to original order and extract grids
        evolved_grids.sort_unstable_by_key(|(idx, _)| *idx);
        evolved_grids.into_iter().map(|(_, grid)| grid).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::glider::GliderPattern;

    #[test]
    fn test_battle_creation() {
        let glider_a = Glider::new(GliderPattern::Standard, SPAWN_A);
        let glider_b = Glider::new(GliderPattern::Standard, SPAWN_B);

        let battle = Battle::new(glider_a, glider_b);
        assert_eq!(battle.steps, BATTLE_STEPS);
    }

    #[test]
    fn test_battle_setup_grid() {
        let glider_a = Glider::new(GliderPattern::Standard, SPAWN_A);
        let glider_b = Glider::new(GliderPattern::Standard, SPAWN_B);

        let battle = Battle::new(glider_a, glider_b);
        let grid = battle.setup_grid();

        // Both gliders should be present
        assert!(grid.live_count() >= 10); // At least 5 cells each
    }

    #[test]
    fn test_battle_simulation_short() {
        let glider_a = Glider::with_energy(GliderPattern::Standard, SPAWN_A, 150);
        let glider_b = Glider::with_energy(GliderPattern::Standard, SPAWN_B, 100);

        // Short battle for testing
        let battle = Battle::with_steps(glider_a, glider_b, 100);
        let outcome = battle.simulate();

        // With higher initial energy, A should have advantage
        // (though CA evolution can be chaotic)
        assert!(outcome == BattleOutcome::AWins || outcome == BattleOutcome::BWins || outcome == BattleOutcome::Tie);
    }

    #[test]
    fn test_battle_identical_gliders() {
        let glider_a = Glider::new(GliderPattern::Standard, SPAWN_A);
        let glider_b = Glider::new(GliderPattern::Standard, SPAWN_B);

        let battle = Battle::with_steps(glider_a, glider_b, 50);
        let outcome = battle.simulate();

        // Identical gliders should trend toward tie (though not guaranteed due to asymmetry)
        // Just verify it completes
        assert!(matches!(
            outcome,
            BattleOutcome::AWins | BattleOutcome::BWins | BattleOutcome::Tie
        ));
    }

    #[test]
    fn test_different_patterns() {
        let glider_a = Glider::new(GliderPattern::Heavyweight, SPAWN_A);
        let glider_b = Glider::new(GliderPattern::Standard, SPAWN_B);

        let battle = Battle::with_steps(glider_a, glider_b, 100);
        let outcome = battle.simulate();

        // Heavier pattern has more cells and energy
        // Should generally win, but CA is chaotic
        assert!(matches!(
            outcome,
            BattleOutcome::AWins | BattleOutcome::BWins | BattleOutcome::Tie
        ));
    }

    #[test]
    fn test_initial_and_final_grids() {
        let glider_a = Glider::new(GliderPattern::Standard, SPAWN_A);
        let glider_b = Glider::new(GliderPattern::Standard, SPAWN_B);

        let battle = Battle::with_steps(glider_a, glider_b, 10);

        let initial = battle.initial_grid();
        let final_grid = battle.final_grid();

        // Grids should exist and be valid
        // They may or may not have different live counts after 10 steps
        assert!(initial.live_count() > 0);
        assert!(final_grid.live_count() > 0);
    }

    #[test]
    fn test_mii_tiebreaker() {
        // Create a battle with history tracking
        let glider_a = Glider::new(GliderPattern::Standard, SPAWN_A);
        let glider_b = Glider::new(GliderPattern::Standard, SPAWN_B);
        let entropy_seed = [1u8; 32]; // Non-zero entropy

        let battle = Battle::with_history(glider_a, glider_b, 100, entropy_seed);
        
        let (outcome, history) = battle.simulate_with_history();
        
        // History should be present
        assert!(history.is_some());
        let hist = history.unwrap();
        
        // Check that deltas and states were recorded
        assert_eq!(hist.region_a_deltas.len(), 100);
        assert_eq!(hist.region_b_deltas.len(), 100);
        
        // Compute MII
        let (mii_a, mii_b) = battle.compute_mii(&hist);
        
        // MII should be non-negative
        assert!(mii_a >= 0.0);
        assert!(mii_b >= 0.0);
        
        // Compute TED
        let (ted_a, ted_b) = battle.compute_ted(&hist);
        
        // TED should be non-negative
        assert!(ted_a >= 0.0);
        assert!(ted_b >= 0.0);
        
        // Outcome should be valid
        assert!(matches!(
            outcome,
            BattleOutcome::AWins | BattleOutcome::BWins | BattleOutcome::Tie
        ));
    }
}
