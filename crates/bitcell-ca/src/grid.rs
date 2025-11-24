//! CA Grid implementation - 1024×1024 toroidal grid with 8-bit cell states

use serde::{Deserialize, Serialize};

/// Grid size constant
pub const GRID_SIZE: usize = 1024;

/// Position on the grid
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Position {
    pub x: usize,
    pub y: usize,
}

impl Position {
    pub fn new(x: usize, y: usize) -> Self {
        Self { x, y }
    }

    /// Wrap position to handle toroidal topology
    pub fn wrap(&self) -> Self {
        Self {
            x: self.x % GRID_SIZE,
            y: self.y % GRID_SIZE,
        }
    }

    /// Get 8 neighbors (Moore neighborhood) with toroidal wrapping
    pub fn neighbors(&self) -> [Position; 8] {
        let x = self.x as isize;
        let y = self.y as isize;
        let size = GRID_SIZE as isize;

        [
            Position::new(((x - 1 + size) % size) as usize, ((y - 1 + size) % size) as usize),
            Position::new(((x - 1 + size) % size) as usize, (y % size) as usize),
            Position::new(((x - 1 + size) % size) as usize, ((y + 1) % size) as usize),
            Position::new((x % size) as usize, ((y - 1 + size) % size) as usize),
            Position::new((x % size) as usize, ((y + 1) % size) as usize),
            Position::new(((x + 1) % size) as usize, ((y - 1 + size) % size) as usize),
            Position::new(((x + 1) % size) as usize, (y % size) as usize),
            Position::new(((x + 1) % size) as usize, ((y + 1) % size) as usize),
        ]
    }
}

/// Cell state with energy
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Cell {
    /// Cell state: 0 = dead, 1-255 = alive with energy
    pub state: u8,
}

impl Cell {
    pub fn dead() -> Self {
        Self { state: 0 }
    }

    pub fn alive(energy: u8) -> Self {
        Self {
            state: energy.max(1),
        }
    }

    pub fn is_alive(&self) -> bool {
        self.state > 0
    }

    pub fn energy(&self) -> u8 {
        self.state
    }
}

/// CA Grid
#[derive(Clone, Serialize, Deserialize)]
pub struct Grid {
    /// Flat array of cells (row-major order)
    pub cells: Vec<Cell>,
}

impl Grid {
    /// Create an empty grid
    pub fn new() -> Self {
        Self {
            cells: vec![Cell::dead(); GRID_SIZE * GRID_SIZE],
        }
    }

    /// Get cell at position
    pub fn get(&self, pos: Position) -> Cell {
        let pos = pos.wrap();
        self.cells[pos.y * GRID_SIZE + pos.x]
    }

    /// Set cell at position
    pub fn set(&mut self, pos: Position, cell: Cell) {
        let pos = pos.wrap();
        self.cells[pos.y * GRID_SIZE + pos.x] = cell;
    }

    /// Count live cells
    pub fn live_count(&self) -> usize {
        self.cells.iter().filter(|c| c.is_alive()).count()
    }

    /// Total energy in grid
    pub fn total_energy(&self) -> u64 {
        self.cells.iter().map(|c| c.energy() as u64).sum()
    }

    /// Get cells in a region
    pub fn region(&self, top_left: Position, width: usize, height: usize) -> Vec<Vec<Cell>> {
        let mut result = Vec::new();
        for dy in 0..height {
            let mut row = Vec::new();
            for dx in 0..width {
                let pos = Position::new(top_left.x + dx, top_left.y + dy);
                row.push(self.get(pos));
            }
            result.push(row);
        }
        result
    }

    /// Set a pattern at a position
    pub fn set_pattern(&mut self, top_left: Position, pattern: &[Vec<Cell>]) {
        for (dy, row) in pattern.iter().enumerate() {
            for (dx, &cell) in row.iter().enumerate() {
                let pos = Position::new(top_left.x + dx, top_left.y + dy);
                self.set(pos, cell);
            }
        }
    }

    /// Clear the grid
    pub fn clear(&mut self) {
        for cell in &mut self.cells {
            *cell = Cell::dead();
        }
    }

    /// Get a downsampled view of the grid for visualization.
    /// 
    /// Uses max pooling to downsample the grid: divides the grid into blocks
    /// and returns the maximum energy value in each block. This is useful for
    /// visualizing large grids at lower resolutions.
    /// 
    /// # Arguments
    /// * `target_size` - The desired output grid size (must be > 0 and <= GRID_SIZE)
    /// 
    /// # Returns
    /// A 2D vector of size `target_size × target_size` containing max energy values.
    /// 
    /// # Panics
    /// Panics if `target_size` is 0 or greater than `GRID_SIZE`.
    /// 
    /// # Note
    /// When `GRID_SIZE` is not evenly divisible by `target_size`, some cells near
    /// the edges may not be sampled. For example, with `GRID_SIZE=1024` and
    /// `target_size=100`, `block_size=10`, so only cells from indices 0-999 are
    /// sampled, leaving rows/columns 1000-1023 unsampled. This is acceptable for
    /// visualization purposes where approximate representation is sufficient.
    pub fn downsample(&self, target_size: usize) -> Vec<Vec<u8>> {
        if target_size == 0 || target_size > GRID_SIZE {
            panic!("target_size must be between 1 and {}", GRID_SIZE);
        }

        let block_size = GRID_SIZE / target_size;
        let mut result = vec![vec![0u8; target_size]; target_size];

        for y in 0..target_size {
            for x in 0..target_size {
                let mut max_energy = 0u8;
                // Sample block
                for by in 0..block_size {
                    for bx in 0..block_size {
                        let pos = Position::new(x * block_size + bx, y * block_size + by);
                        max_energy = max_energy.max(self.get(pos).energy());
                    }
                }
                result[y][x] = max_energy;
            }
        }

        result
    }
}

impl Default for Grid {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_grid_creation() {
        let grid = Grid::new();
        assert_eq!(grid.live_count(), 0);
        assert_eq!(grid.total_energy(), 0);
    }

    #[test]
    fn test_cell_set_get() {
        let mut grid = Grid::new();
        let pos = Position::new(10, 20);
        let cell = Cell::alive(100);

        grid.set(pos, cell);
        assert_eq!(grid.get(pos), cell);
    }

    #[test]
    fn test_toroidal_wrap() {
        let mut grid = Grid::new();
        let pos = Position::new(GRID_SIZE - 1, GRID_SIZE - 1);
        let cell = Cell::alive(50);

        grid.set(pos, cell);

        // Access through wraparound
        let wrapped = Position::new(2 * GRID_SIZE - 1, 2 * GRID_SIZE - 1);
        assert_eq!(grid.get(wrapped), cell);
    }

    #[test]
    fn test_neighbors() {
        let pos = Position::new(10, 10);
        let neighbors = pos.neighbors();
        assert_eq!(neighbors.len(), 8);

        // Check that all neighbors are distinct
        for i in 0..8 {
            for j in (i + 1)..8 {
                assert_ne!(neighbors[i], neighbors[j]);
            }
        }
    }

    #[test]
    fn test_neighbors_wraparound() {
        let pos = Position::new(0, 0);
        let neighbors = pos.neighbors();

        // Should wrap around to the opposite side
        assert!(neighbors.iter().any(|n| n.x == GRID_SIZE - 1));
        assert!(neighbors.iter().any(|n| n.y == GRID_SIZE - 1));
    }

    #[test]
    fn test_pattern_placement() {
        let mut grid = Grid::new();
        let pattern = vec![
            vec![Cell::alive(100), Cell::alive(100)],
            vec![Cell::alive(100), Cell::alive(100)],
        ];

        grid.set_pattern(Position::new(5, 5), &pattern);

        assert_eq!(grid.live_count(), 4);
        assert_eq!(grid.get(Position::new(5, 5)), Cell::alive(100));
        assert_eq!(grid.get(Position::new(6, 6)), Cell::alive(100));
    }
}
