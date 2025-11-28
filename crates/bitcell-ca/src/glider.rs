//! Glider patterns for tournament combat
//!
//! Standard patterns that miners can submit for battles.

use crate::grid::{Cell, Position};
use serde::{Deserialize, Serialize};

/// Known glider patterns
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GliderPattern {
    /// Standard Conway glider
    ///  #
    ///   #
    /// ###
    Standard,

    /// Lightweight spaceship (LWSS)
    ///  #  #
    /// #
    /// #   #
    /// ####
    Lightweight,

    /// Middleweight spaceship (MWSS)
    ///   #
    /// #   #
    /// #
    /// #    #
    /// #####
    Middleweight,

    /// Heavyweight spaceship (HWSS)
    ///   ##
    /// #    #
    /// #
    /// #     #
    /// ######
    Heavyweight,
}

impl GliderPattern {
    /// Get the pattern as a 2D array of cells
    pub fn cells(&self, energy: u8) -> Vec<Vec<Cell>> {
        let alive = Cell::alive(energy);
        let dead = Cell::dead();

        match self {
            GliderPattern::Standard => vec![
                vec![dead, alive, dead],
                vec![dead, dead, alive],
                vec![alive, alive, alive],
            ],

            GliderPattern::Lightweight => vec![
                vec![dead, alive, dead, dead, alive],
                vec![alive, dead, dead, dead, dead],
                vec![alive, dead, dead, dead, alive],
                vec![alive, alive, alive, alive, dead],
            ],

            GliderPattern::Middleweight => vec![
                vec![dead, dead, dead, alive, dead, dead],
                vec![dead, alive, dead, dead, dead, alive],
                vec![alive, dead, dead, dead, dead, dead],
                vec![alive, dead, dead, dead, dead, alive],
                vec![alive, alive, alive, alive, alive, dead],
            ],

            GliderPattern::Heavyweight => vec![
                vec![dead, dead, dead, alive, alive, dead, dead],
                vec![dead, alive, dead, dead, dead, dead, alive],
                vec![alive, dead, dead, dead, dead, dead, dead],
                vec![alive, dead, dead, dead, dead, dead, alive],
                vec![alive, alive, alive, alive, alive, alive, dead],
            ],
        }
    }

    /// Get pattern dimensions (width, height)
    pub fn dimensions(&self) -> (usize, usize) {
        let cells = self.cells(1);
        (cells[0].len(), cells.len())
    }

    /// Get initial energy for this pattern
    pub fn default_energy(&self) -> u8 {
        match self {
            GliderPattern::Standard => 100,
            GliderPattern::Lightweight => 120,
            GliderPattern::Middleweight => 140,
            GliderPattern::Heavyweight => 160,
        }
    }

    /// Convert pattern to bytes for hashing
    pub fn to_bytes(&self) -> Vec<u8> {
        match self {
            GliderPattern::Standard => b"Standard".to_vec(),
            GliderPattern::Lightweight => b"Lightweight".to_vec(),
            GliderPattern::Middleweight => b"Middleweight".to_vec(),
            GliderPattern::Heavyweight => b"Heavyweight".to_vec(),
        }
    }

    /// List all available patterns
    pub fn all() -> Vec<GliderPattern> {
        vec![
            GliderPattern::Standard,
            GliderPattern::Lightweight,
            GliderPattern::Middleweight,
            GliderPattern::Heavyweight,
        ]
    }
}

/// A glider instance with position and pattern
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Glider {
    pub pattern: GliderPattern,
    pub position: Position,
    pub energy: u8,
}

impl Glider {
    pub fn new(pattern: GliderPattern, position: Position) -> Self {
        Self {
            pattern,
            position,
            energy: pattern.default_energy(),
        }
    }

    pub fn with_energy(pattern: GliderPattern, position: Position, energy: u8) -> Self {
        Self {
            pattern,
            position,
            energy,
        }
    }

    /// Get the cells for this glider
    pub fn cells(&self) -> Vec<Vec<Cell>> {
        self.pattern.cells(self.energy)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_standard_glider_dimensions() {
        let pattern = GliderPattern::Standard;
        assert_eq!(pattern.dimensions(), (3, 3));
    }

    #[test]
    fn test_glider_cell_count() {
        let pattern = GliderPattern::Standard;
        let cells = pattern.cells(100);

        let alive_count: usize = cells
            .iter()
            .map(|row| row.iter().filter(|c| c.is_alive()).count())
            .sum();

        assert_eq!(alive_count, 5); // Standard glider has 5 live cells
    }

    #[test]
    fn test_all_patterns() {
        let patterns = GliderPattern::all();
        assert_eq!(patterns.len(), 4);

        for pattern in patterns {
            let cells = pattern.cells(100);
            assert!(!cells.is_empty());
            assert!(!cells[0].is_empty());
        }
    }

    #[test]
    fn test_glider_creation() {
        let glider = Glider::new(GliderPattern::Standard, Position::new(10, 10));
        assert_eq!(glider.energy, 100);
        assert_eq!(glider.position, Position::new(10, 10));
    }

    #[test]
    fn test_glider_with_custom_energy() {
        let glider = Glider::with_energy(
            GliderPattern::Lightweight,
            Position::new(20, 20),
            200,
        );
        assert_eq!(glider.energy, 200);
    }

    #[test]
    fn test_lightweight_spaceship() {
        let pattern = GliderPattern::Lightweight;
        let cells = pattern.cells(100);

        let alive_count: usize = cells
            .iter()
            .map(|row| row.iter().filter(|c| c.is_alive()).count())
            .sum();

        assert_eq!(alive_count, 9); // LWSS has 9 live cells
    }
}
