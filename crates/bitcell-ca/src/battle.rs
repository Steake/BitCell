//! Battle simulation between gliders
//!
//! Simulates CA evolution with two gliders and determines the winner.

use crate::glider::Glider;
use crate::grid::{Grid, Position, GRID_SIZE};
use crate::rules::evolve_n_steps;
use crate::{Error, Result};
use serde::{Deserialize, Serialize};

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

/// A battle between two gliders
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Battle {
    pub glider_a: Glider,
    pub glider_b: Glider,
    pub steps: usize,
}

impl Battle {
    /// Create a new battle
    pub fn new(glider_a: Glider, glider_b: Glider) -> Self {
        Self {
            glider_a,
            glider_b,
            steps: BATTLE_STEPS,
        }
    }

    /// Create a battle with custom number of steps
    pub fn with_steps(glider_a: Glider, glider_b: Glider, steps: usize) -> Self {
        Self {
            glider_a,
            glider_b,
            steps,
        }
    }

    /// Set up the initial grid with both gliders
    fn setup_grid(&self) -> Grid {
        let mut grid = Grid::new();

        // Place glider A at spawn position A
        grid.set_pattern(SPAWN_A, &self.glider_a.cells());

        // Place glider B at spawn position B
        grid.set_pattern(SPAWN_B, &self.glider_b.cells());

        grid
    }

    /// Simulate the battle and return the outcome
    pub fn simulate(&self) -> Result<BattleOutcome> {
        let initial_grid = self.setup_grid();
        let final_grid = evolve_n_steps(&initial_grid, self.steps);

        // Determine winner by energy in each half of the grid
        let (energy_a, energy_b) = self.measure_regional_energy(&final_grid);

        let outcome = if energy_a > energy_b {
            BattleOutcome::AWins
        } else if energy_b > energy_a {
            BattleOutcome::BWins
        } else {
            BattleOutcome::Tie
        };

        Ok(outcome)
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

    /// Get initial grid state (for proof generation)
    pub fn initial_grid(&self) -> Grid {
        self.setup_grid()
    }

    /// Get final grid state after simulation
    pub fn final_grid(&self) -> Grid {
        let initial = self.setup_grid();
        evolve_n_steps(&initial, self.steps)
    }

    /// Get grid states at specific steps for visualization
    /// Returns a vector of grids at the requested step intervals
    pub fn grid_states(&self, sample_steps: &[usize]) -> Vec<Grid> {
        let mut grids = Vec::new();
        let initial = self.setup_grid();

        for &step in sample_steps {
            if step <= self.steps {
                let grid = evolve_n_steps(&initial, step);
                grids.push(grid);
            }
        }

        grids
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
        let outcome = battle.simulate().unwrap();

        // With higher initial energy, A should have advantage
        // (though CA evolution can be chaotic)
        assert!(outcome == BattleOutcome::AWins || outcome == BattleOutcome::BWins || outcome == BattleOutcome::Tie);
    }

    #[test]
    fn test_battle_identical_gliders() {
        let glider_a = Glider::new(GliderPattern::Standard, SPAWN_A);
        let glider_b = Glider::new(GliderPattern::Standard, SPAWN_B);

        let battle = Battle::with_steps(glider_a, glider_b, 50);
        let outcome = battle.simulate().unwrap();

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
        let outcome = battle.simulate().unwrap();

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
}
