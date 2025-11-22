//! CA evolution rules - Conway-like with energy
//!
//! Rules:
//! - Live cells with 2-3 neighbors survive
//! - Dead cells with exactly 3 neighbors become alive
//! - New cells inherit average energy from neighbors
//! - Cells that die lose their energy

use crate::grid::{Cell, Grid, Position};
use rayon::prelude::*;

/// Evolve a cell based on its neighbors (Conway-like rules with energy)
pub fn evolve_cell(cell: Cell, neighbors: &[Cell; 8]) -> Cell {
    let live_neighbors: Vec<&Cell> = neighbors.iter().filter(|c| c.is_alive()).collect();
    let live_count = live_neighbors.len();

    if cell.is_alive() {
        // Survival rules
        if live_count == 2 || live_count == 3 {
            // Cell survives, keeps its energy
            cell
        } else {
            // Cell dies (underpopulation or overpopulation)
            Cell::dead()
        }
    } else {
        // Birth rules
        if live_count == 3 {
            // Cell becomes alive with average energy of neighbors
            let avg_energy = if live_neighbors.is_empty() {
                1
            } else {
                let total: u32 = live_neighbors.iter().map(|c| c.energy() as u32).sum();
                ((total / live_neighbors.len() as u32) as u8).max(1)
            };
            Cell::alive(avg_energy)
        } else {
            // Cell stays dead
            Cell::dead()
        }
    }
}

/// Evolve the entire grid one step
pub fn evolve_grid(grid: &Grid) -> Grid {
    let mut new_grid = Grid::new();

    // Use parallel processing for large grid
    let size = crate::grid::GRID_SIZE;
    let cells: Vec<_> = (0..size)
        .into_par_iter()
        .flat_map(|y| {
            (0..size)
                .map(|x| {
                    let pos = Position::new(x, y);
                    let cell = grid.get(pos);
                    let neighbor_positions = pos.neighbors();
                    let neighbors = [
                        grid.get(neighbor_positions[0]),
                        grid.get(neighbor_positions[1]),
                        grid.get(neighbor_positions[2]),
                        grid.get(neighbor_positions[3]),
                        grid.get(neighbor_positions[4]),
                        grid.get(neighbor_positions[5]),
                        grid.get(neighbor_positions[6]),
                        grid.get(neighbor_positions[7]),
                    ];

                    (pos, evolve_cell(cell, &neighbors))
                })
                .collect::<Vec<_>>()
        })
        .collect();

    for (pos, cell) in cells {
        new_grid.set(pos, cell);
    }

    new_grid
}

/// Evolve grid for N steps
pub fn evolve_n_steps(grid: &Grid, steps: usize) -> Grid {
    let mut current = grid.clone();
    for _ in 0..steps {
        current = evolve_grid(&current);
    }
    current
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dead_cell_stays_dead() {
        let cell = Cell::dead();
        let neighbors = [Cell::dead(); 8];
        let result = evolve_cell(cell, &neighbors);
        assert!(!result.is_alive());
    }

    #[test]
    fn test_live_cell_survives_with_2_neighbors() {
        let cell = Cell::alive(100);
        let mut neighbors = [Cell::dead(); 8];
        neighbors[0] = Cell::alive(100);
        neighbors[1] = Cell::alive(100);

        let result = evolve_cell(cell, &neighbors);
        assert!(result.is_alive());
        assert_eq!(result.energy(), 100);
    }

    #[test]
    fn test_live_cell_survives_with_3_neighbors() {
        let cell = Cell::alive(100);
        let mut neighbors = [Cell::dead(); 8];
        neighbors[0] = Cell::alive(100);
        neighbors[1] = Cell::alive(100);
        neighbors[2] = Cell::alive(100);

        let result = evolve_cell(cell, &neighbors);
        assert!(result.is_alive());
    }

    #[test]
    fn test_live_cell_dies_underpopulation() {
        let cell = Cell::alive(100);
        let mut neighbors = [Cell::dead(); 8];
        neighbors[0] = Cell::alive(100);

        let result = evolve_cell(cell, &neighbors);
        assert!(!result.is_alive());
    }

    #[test]
    fn test_live_cell_dies_overpopulation() {
        let cell = Cell::alive(100);
        let neighbors = [Cell::alive(100); 8];

        let result = evolve_cell(cell, &neighbors);
        assert!(!result.is_alive());
    }

    #[test]
    fn test_dead_cell_born_with_3_neighbors() {
        let cell = Cell::dead();
        let mut neighbors = [Cell::dead(); 8];
        neighbors[0] = Cell::alive(90);
        neighbors[1] = Cell::alive(100);
        neighbors[2] = Cell::alive(110);

        let result = evolve_cell(cell, &neighbors);
        assert!(result.is_alive());

        // Average energy should be (90 + 100 + 110) / 3 = 100
        assert_eq!(result.energy(), 100);
    }

    #[test]
    fn test_grid_evolution() {
        let mut grid = Grid::new();

        // Create a simple blinker pattern
        // ###
        grid.set(Position::new(10, 10), Cell::alive(100));
        grid.set(Position::new(11, 10), Cell::alive(100));
        grid.set(Position::new(12, 10), Cell::alive(100));

        assert_eq!(grid.live_count(), 3);

        // Evolve one step - should rotate to vertical
        let grid2 = evolve_grid(&grid);
        assert_eq!(grid2.live_count(), 3);

        // Evolve again - should rotate back to horizontal
        let grid3 = evolve_grid(&grid2);
        assert_eq!(grid3.live_count(), 3);
    }

    #[test]
    fn test_evolve_n_steps() {
        let mut grid = Grid::new();

        // Stable block pattern
        // ##
        // ##
        grid.set(Position::new(10, 10), Cell::alive(100));
        grid.set(Position::new(11, 10), Cell::alive(100));
        grid.set(Position::new(10, 11), Cell::alive(100));
        grid.set(Position::new(11, 11), Cell::alive(100));

        let evolved = evolve_n_steps(&grid, 10);
        
        // Block should remain stable
        assert_eq!(evolved.live_count(), 4);
    }
}
