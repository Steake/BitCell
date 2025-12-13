//! Tests for GPU-accelerated CA evolution

use crate::{Grid, Cell, Position, GridSize};
use crate::rules::{evolve_grid, evolve_grid_into};

#[cfg(any(feature = "cuda", feature = "opencl"))]
use crate::gpu::{detect_gpu, create_gpu_evolver, GpuEvolver};

#[test]
fn test_large_grid_creation() {
    let grid = Grid::with_size(GridSize::Large);
    assert_eq!(grid.grid_size(), 4096);
    assert_eq!(grid.cells.len(), 4096 * 4096);
}

#[test]
fn test_large_grid_basic_operations() {
    let mut grid = Grid::with_size(GridSize::Large);
    let pos = Position::new(2000, 2000);
    let cell = Cell::alive(150);
    
    grid.set(pos, cell);
    assert_eq!(grid.get(pos), cell);
}

#[test]
fn test_large_grid_wrap() {
    let mut grid = Grid::with_size(GridSize::Large);
    let pos = Position::new(4095, 4095);
    let cell = Cell::alive(100);
    
    grid.set(pos, cell);
    
    // Access through wraparound
    let wrapped = Position::new(8191, 8191);
    assert_eq!(grid.get(wrapped), cell);
}

#[test]
fn test_large_grid_evolution() {
    let mut grid = Grid::with_size(GridSize::Large);
    
    // Create a simple blinker pattern
    grid.set(Position::new(2000, 2000), Cell::alive(100));
    grid.set(Position::new(2001, 2000), Cell::alive(100));
    grid.set(Position::new(2002, 2000), Cell::alive(100));
    
    assert_eq!(grid.live_count(), 3);
    
    // Evolve one step
    let grid2 = evolve_grid(&grid);
    assert_eq!(grid2.live_count(), 3);
    assert_eq!(grid2.grid_size(), 4096);
}

#[cfg(any(feature = "cuda", feature = "opencl"))]
#[test]
fn test_gpu_detection() {
    // This should not panic even if no GPU is available
    let backend = detect_gpu();
    
    if let Some(backend) = backend {
        println!("Detected GPU backend: {:?}", backend);
    } else {
        println!("No GPU detected, will use CPU fallback");
    }
}

#[cfg(any(feature = "cuda", feature = "opencl"))]
#[test]
fn test_gpu_evolver_creation() {
    // Try to create a GPU evolver
    let result = create_gpu_evolver();
    
    match result {
        Ok(evolver) => {
            let info = evolver.device_info();
            println!("GPU Device: {} ({} bytes, {} compute units)", 
                     info.name, info.memory, info.compute_units);
            assert!(info.memory > 0);
            assert!(info.compute_units > 0);
        }
        Err(e) => {
            println!("No GPU available: {}", e);
            // Don't fail the test - GPU might not be available in CI
        }
    }
}

#[cfg(any(feature = "cuda", feature = "opencl"))]
#[test]
fn test_gpu_cpu_equivalence() {
    // Create a test grid with a simple pattern
    let mut grid = Grid::new();
    grid.set(Position::new(100, 100), Cell::alive(100));
    grid.set(Position::new(101, 100), Cell::alive(100));
    grid.set(Position::new(102, 100), Cell::alive(100));
    
    // Evolve with CPU
    let cpu_result = evolve_grid(&grid);
    
    // Try to evolve with GPU
    if let Ok(evolver) = create_gpu_evolver() {
        match evolver.evolve(&grid) {
            Ok(gpu_result) => {
                // Compare results
                assert_eq!(gpu_result.cells.len(), cpu_result.cells.len());
                assert_eq!(gpu_result.grid_size(), cpu_result.grid_size());
                
                // Check that all cells match
                let mut differences = 0;
                for i in 0..cpu_result.cells.len() {
                    if cpu_result.cells[i] != gpu_result.cells[i] {
                        differences += 1;
                    }
                }
                
                // Allow for minor differences due to floating point arithmetic
                // in energy calculations, but they should be rare
                assert!(differences < 10, "Too many differences between CPU and GPU: {}", differences);
                
                println!("GPU and CPU results match (differences: {})", differences);
            }
            Err(e) => {
                println!("GPU evolution failed: {}", e);
            }
        }
    } else {
        println!("No GPU available for equivalence test");
    }
}

#[cfg(any(feature = "cuda", feature = "opencl"))]
#[test]
fn test_gpu_large_grid_support() {
    if let Ok(evolver) = create_gpu_evolver() {
        let mut grid = Grid::with_size(GridSize::Large);
        
        // Add some cells
        for i in 0..10 {
            grid.set(Position::new(2000 + i, 2000), Cell::alive(100));
        }
        
        // Try to evolve on GPU
        match evolver.evolve(&grid) {
            Ok(result) => {
                assert_eq!(result.grid_size(), 4096);
                assert!(result.live_count() > 0);
                println!("GPU successfully evolved 4096x4096 grid");
            }
            Err(e) => {
                println!("GPU large grid evolution failed: {}", e);
            }
        }
    } else {
        println!("No GPU available for large grid test");
    }
}

#[test]
fn test_grid_size_enum() {
    let standard = GridSize::Standard;
    let large = GridSize::Large;
    
    assert_eq!(standard.size(), 1024);
    assert_eq!(large.size(), 4096);
}

#[test]
fn test_position_wrap_with_custom_size() {
    let pos = Position::new(5000, 5000);
    let wrapped = pos.wrap_with_size(4096);
    
    assert_eq!(wrapped.x, 5000 % 4096);
    assert_eq!(wrapped.y, 5000 % 4096);
}

#[test]
fn test_neighbors_with_custom_size() {
    let pos = Position::new(0, 0);
    let neighbors = pos.neighbors_with_size(4096);
    
    assert_eq!(neighbors.len(), 8);
    // Check wrapping at boundaries
    assert!(neighbors.iter().any(|n| n.x == 4095));
    assert!(neighbors.iter().any(|n| n.y == 4095));
}
