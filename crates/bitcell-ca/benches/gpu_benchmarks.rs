use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId, BenchmarkGroup};
use criterion::measurement::WallTime;
use bitcell_ca::{Grid, Cell, Position, GridSize, GRID_SIZE, LARGE_GRID_SIZE};
use bitcell_ca::rules::evolve_grid;

#[cfg(any(feature = "cuda", feature = "opencl"))]
use bitcell_ca::gpu::{create_gpu_evolver, GpuEvolver};

fn setup_grid(size: usize, density: f32) -> Grid {
    let mut grid = if size == LARGE_GRID_SIZE {
        Grid::with_size(GridSize::Large)
    } else {
        Grid::new()
    };
    
    // Add random patterns
    for i in 0..(size as f32 * size as f32 * density) as usize {
        let x = (i * 7) % size;
        let y = (i * 13) % size;
        grid.set(Position::new(x, y), Cell::alive(128));
    }
    
    grid
}

fn bench_cpu_evolution(c: &mut Criterion) {
    let mut group = c.benchmark_group("cpu_evolution");
    
    // Benchmark standard grid
    let grid = setup_grid(GRID_SIZE, 0.01);
    group.bench_function("cpu_1024x1024", |b| {
        b.iter(|| {
            let g = grid.clone();
            black_box(evolve_grid(&g))
        });
    });
    
    // Benchmark large grid
    let large_grid = setup_grid(LARGE_GRID_SIZE, 0.001);
    group.bench_function("cpu_4096x4096", |b| {
        b.iter(|| {
            let g = large_grid.clone();
            black_box(evolve_grid(&g))
        });
    });
    
    group.finish();
}

#[cfg(any(feature = "cuda", feature = "opencl"))]
fn bench_gpu_evolution(c: &mut Criterion) {
    if let Ok(evolver) = create_gpu_evolver() {
        let mut group = c.benchmark_group("gpu_evolution");
        
        let device_info = evolver.device_info();
        println!("\nGPU Device: {} ({} bytes, {} compute units)", 
                 device_info.name, device_info.memory, device_info.compute_units);
        
        // Benchmark standard grid
        let grid = setup_grid(GRID_SIZE, 0.01);
        group.bench_function("gpu_1024x1024", |b| {
            b.iter(|| {
                black_box(evolver.evolve(&grid).unwrap())
            });
        });
        
        // Benchmark large grid
        let large_grid = setup_grid(LARGE_GRID_SIZE, 0.001);
        group.bench_function("gpu_4096x4096", |b| {
            b.iter(|| {
                black_box(evolver.evolve(&large_grid).unwrap())
            });
        });
        
        group.finish();
    } else {
        println!("\nNo GPU available - skipping GPU benchmarks");
    }
}

#[cfg(any(feature = "cuda", feature = "opencl"))]
fn bench_gpu_vs_cpu(c: &mut Criterion) {
    if let Ok(evolver) = create_gpu_evolver() {
        let mut group = c.benchmark_group("gpu_vs_cpu");
        
        // Test different grid densities
        for density in [0.001, 0.01, 0.1].iter() {
            let grid = setup_grid(GRID_SIZE, *density);
            
            group.bench_with_input(
                BenchmarkId::new("cpu", format!("density_{}", density)),
                &grid,
                |b, g| {
                    b.iter(|| black_box(evolve_grid(g)))
                }
            );
            
            group.bench_with_input(
                BenchmarkId::new("gpu", format!("density_{}", density)),
                &grid,
                |b, g| {
                    b.iter(|| black_box(evolver.evolve(g).unwrap()))
                }
            );
        }
        
        group.finish();
    }
}

fn bench_multi_step_evolution(c: &mut Criterion) {
    let mut group = c.benchmark_group("multi_step");
    
    let grid = setup_grid(GRID_SIZE, 0.01);
    
    for steps in [10, 100, 1000].iter() {
        group.bench_with_input(
            BenchmarkId::new("cpu", steps),
            steps,
            |b, &steps| {
                b.iter(|| {
                    let mut current = grid.clone();
                    for _ in 0..steps {
                        current = evolve_grid(&current);
                    }
                    black_box(current)
                });
            }
        );
        
        #[cfg(any(feature = "cuda", feature = "opencl"))]
        if let Ok(evolver) = create_gpu_evolver() {
            group.bench_with_input(
                BenchmarkId::new("gpu", steps),
                steps,
                |b, &steps| {
                    b.iter(|| {
                        let mut current = grid.clone();
                        for _ in 0..steps {
                            current = evolver.evolve(&current).unwrap();
                        }
                        black_box(current)
                    });
                }
            );
        }
    }
    
    group.finish();
}

fn bench_grid_size_comparison(c: &mut Criterion) {
    let mut group = c.benchmark_group("grid_sizes");
    
    // Standard grid
    let standard = setup_grid(GRID_SIZE, 0.01);
    group.bench_function("1024x1024", |b| {
        b.iter(|| {
            let g = standard.clone();
            black_box(evolve_grid(&g))
        });
    });
    
    // Large grid (lower density to keep benchmark time reasonable)
    let large = setup_grid(LARGE_GRID_SIZE, 0.001);
    group.bench_function("4096x4096", |b| {
        b.iter(|| {
            let g = large.clone();
            black_box(evolve_grid(&g))
        });
    });
    
    group.finish();
}

criterion_group!(
    benches,
    bench_cpu_evolution,
    bench_grid_size_comparison,
    bench_multi_step_evolution,
);

#[cfg(any(feature = "cuda", feature = "opencl"))]
criterion_group!(
    gpu_benches,
    bench_gpu_evolution,
    bench_gpu_vs_cpu,
);

#[cfg(any(feature = "cuda", feature = "opencl"))]
criterion_main!(benches, gpu_benches);

#[cfg(not(any(feature = "cuda", feature = "opencl")))]
criterion_main!(benches);
