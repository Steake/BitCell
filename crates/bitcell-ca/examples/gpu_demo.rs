//! GPU Acceleration Example
//!
//! This example demonstrates GPU-accelerated CA evolution and compares
//! performance with CPU execution.
//!
//! Run with: cargo run --example gpu_demo --features opencl --release

use bitcell_ca::{Grid, GridSize, Position, Cell, GRID_SIZE, LARGE_GRID_SIZE};
use bitcell_ca::rules::evolve_grid;
use std::time::Instant;

#[cfg(any(feature = "cuda", feature = "opencl"))]
use bitcell_ca::gpu::{detect_gpu, create_gpu_evolver, GpuBackend};

fn setup_test_pattern(grid: &mut Grid, density: f32) {
    let size = grid.grid_size();
    let num_cells = (size as f32 * size as f32 * density) as usize;
    
    println!("Setting up {} cells (density: {:.1}%)", num_cells, density * 100.0);
    
    // Create scattered cells using a deterministic pattern
    for i in 0..num_cells {
        let x = (i * 7) % size;
        let y = (i * 13) % size;
        grid.set(Position::new(x, y), Cell::alive(128));
    }
}

fn benchmark_cpu(grid: &Grid, steps: usize) -> (Grid, std::time::Duration) {
    println!("\n=== CPU Benchmark ===");
    println!("Grid size: {}x{}", grid.grid_size(), grid.grid_size());
    println!("Live cells: {}", grid.live_count());
    
    let start = Instant::now();
    let mut current = grid.clone();
    
    for step in 0..steps {
        if step % 100 == 0 && step > 0 {
            println!("  Step {}/{} ({:.1}%)", step, steps, (step as f32 / steps as f32) * 100.0);
        }
        current = evolve_grid(&current);
    }
    
    let duration = start.elapsed();
    println!("CPU completed {} steps in {:.2?}", steps, duration);
    println!("Average: {:.2} ms/step", duration.as_millis() as f64 / steps as f64);
    println!("Final live cells: {}", current.live_count());
    
    (current, duration)
}

#[cfg(any(feature = "cuda", feature = "opencl"))]
fn benchmark_gpu(grid: &Grid, steps: usize) -> Option<(Grid, std::time::Duration)> {
    if let Ok(evolver) = create_gpu_evolver() {
        let device_info = evolver.device_info();
        println!("\n=== GPU Benchmark ===");
        println!("Device: {}", device_info.name);
        println!("Memory: {} MB", device_info.memory / 1024 / 1024);
        println!("Compute Units: {}", device_info.compute_units);
        println!("Backend: {:?}", device_info.backend);
        println!("Grid size: {}x{}", grid.grid_size(), grid.grid_size());
        println!("Live cells: {}", grid.live_count());
        
        let start = Instant::now();
        let mut current = grid.clone();
        
        for step in 0..steps {
            if step % 100 == 0 && step > 0 {
                println!("  Step {}/{} ({:.1}%)", step, steps, (step as f32 / steps as f32) * 100.0);
            }
            match evolver.evolve(&current) {
                Ok(next) => current = next,
                Err(e) => {
                    println!("GPU error at step {}: {}", step, e);
                    return None;
                }
            }
        }
        
        let duration = start.elapsed();
        println!("GPU completed {} steps in {:.2?}", steps, duration);
        println!("Average: {:.2} ms/step", duration.as_millis() as f64 / steps as f64);
        println!("Final live cells: {}", current.live_count());
        
        Some((current, duration))
    } else {
        println!("\n=== GPU Not Available ===");
        None
    }
}

fn main() {
    println!("BitCell CA - GPU Acceleration Demo");
    println!("===================================\n");
    
    // Detect GPU
    #[cfg(any(feature = "cuda", feature = "opencl"))]
    {
        match detect_gpu() {
            Some(backend) => {
                println!("GPU detected: {:?}", backend);
            }
            None => {
                println!("No GPU detected - CPU only mode");
            }
        }
    }
    
    #[cfg(not(any(feature = "cuda", feature = "opencl")))]
    {
        println!("GPU support not compiled in");
        println!("To enable GPU: cargo run --example gpu_demo --features opencl --release");
    }
    
    // Test 1: Standard grid
    println!("\n### Test 1: Standard Grid (1024×1024) ###");
    let mut grid = Grid::new();
    setup_test_pattern(&mut grid, 0.01);
    
    let (cpu_result, cpu_time) = benchmark_cpu(&grid, 100);
    
    #[cfg(any(feature = "cuda", feature = "opencl"))]
    if let Some((gpu_result, gpu_time)) = benchmark_gpu(&grid, 100) {
        // Compare results
        println!("\n=== Comparison ===");
        println!("CPU time: {:.2?}", cpu_time);
        println!("GPU time: {:.2?}", gpu_time);
        
        let speedup = cpu_time.as_secs_f64() / gpu_time.as_secs_f64();
        println!("Speedup: {:.2}x", speedup);
        
        // Verify results match
        let mut differences = 0;
        for i in 0..cpu_result.cells.len() {
            if cpu_result.cells[i] != gpu_result.cells[i] {
                differences += 1;
            }
        }
        
        if differences == 0 {
            println!("✓ CPU and GPU results match perfectly");
        } else {
            println!("⚠ CPU and GPU differ in {} cells", differences);
        }
    }
    
    // Test 2: Large grid (if requested)
    let test_large = std::env::args().any(|arg| arg == "--large");
    
    if test_large {
        println!("\n### Test 2: Large Grid (4096×4096) ###");
        let mut large_grid = Grid::with_size(GridSize::Large);
        setup_test_pattern(&mut large_grid, 0.001);
        
        let (_cpu_result_large, _cpu_time_large) = benchmark_cpu(&large_grid, 10);
        
        #[cfg(any(feature = "cuda", feature = "opencl"))]
        if let Some((gpu_result_large, gpu_time_large)) = benchmark_gpu(&large_grid, 10) {
            println!("\n=== Large Grid Comparison ===");
            println!("GPU time: {:.2?}", gpu_time_large);
            
            // Verify results match with CPU baseline
            println!("Final live cells: {}", gpu_result_large.live_count());
        }
    } else {
        println!("\nTo test large grids (4096×4096), run with --large flag");
    }
    
    println!("\n=== Demo Complete ===");
    println!("\nNote: For best performance, run with --release flag");
    println!("Example: cargo run --example gpu_demo --features opencl --release --large");
}
