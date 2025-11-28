//! Cellular Automaton Engine for BitCell
//!
//! Implements the tournament CA system with:
//! - 1024×1024 toroidal grid
//! - Conway-like rules with energy
//! - Glider patterns and collision detection
//! - Battle simulation and outcome determination

pub mod grid;
pub mod rules;
pub mod glider;
pub mod battle;

pub use grid::{Grid, Cell, Position};
pub use glider::{Glider, GliderPattern};
pub use battle::{Battle, BattleOutcome, BattleHistory};

/// Result type for CA operations
pub type Result<T> = std::result::Result<T, Error>;

/// CA-related errors
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Invalid grid position: ({0}, {1})")]
    InvalidPosition(usize, usize),
    
    #[error("Invalid glider pattern")]
    InvalidGlider,
    
    #[error("Battle simulation failed: {0}")]
    BattleError(String),
    
    #[error("Grid operation failed: {0}")]
    GridError(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_imports() {
        // Smoke test
    }
}
