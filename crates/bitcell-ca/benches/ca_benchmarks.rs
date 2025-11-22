use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use bitcell_ca::{Grid, Glider, GliderPattern, Battle, Position};

fn grid_creation_benchmark(c: &mut Criterion) {
    c.bench_function("grid_1024x1024_creation", |b| {
        b.iter(|| Grid::new(black_box(1024), black_box(1024)))
    });
}

fn grid_evolution_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("grid_evolution");
    
    for size in [256, 512, 1024].iter() {
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, &size| {
            let mut grid = Grid::new(size, size);
            // Add some initial patterns
            grid.set_cell(100, 100, 128);
            grid.set_cell(100, 101, 128);
            grid.set_cell(101, 100, 128);
            
            b.iter(|| {
                let mut g = grid.clone();
                g.step();
            });
        });
    }
    group.finish();
}

fn glider_simulation_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("glider_simulation");
    
    let patterns = vec![
        ("Standard", GliderPattern::Standard),
        ("Lightweight", GliderPattern::Lightweight),
        ("Middleweight", GliderPattern::Middleweight),
        ("Heavyweight", GliderPattern::Heavyweight),
    ];
    
    for (name, pattern) in patterns {
        group.bench_with_input(BenchmarkId::from_parameter(name), &pattern, |b, pattern| {
            b.iter(|| {
                let glider = Glider::new(*pattern, Position::new(100, 100));
                let _ = glider.spawn_on_grid(black_box(&mut Grid::new(512, 512)));
            });
        });
    }
    group.finish();
}

fn battle_simulation_benchmark(c: &mut Criterion) {
    c.bench_function("battle_1000_steps", |b| {
        let glider_a = Glider::new(GliderPattern::Heavyweight, Position::new(200, 200));
        let glider_b = Glider::new(GliderPattern::Standard, Position::new(800, 800));
        let battle = Battle::new(glider_a, glider_b);
        
        b.iter(|| {
            let mut b = battle.clone();
            black_box(b.simulate().unwrap())
        });
    });
}

fn parallel_grid_evolution_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("parallel_evolution");
    
    let mut grid = Grid::new(1024, 1024);
    // Add scattered patterns for realistic parallel workload
    for i in 0..10 {
        for j in 0..10 {
            grid.set_cell(i * 100, j * 100, 200);
        }
    }
    
    group.bench_function("sequential_step", |b| {
        b.iter(|| {
            let mut g = grid.clone();
            g.step();
        });
    });
    
    group.bench_function("parallel_step", |b| {
        b.iter(|| {
            let mut g = grid.clone();
            g.step(); // step() uses rayon internally
        });
    });
    
    group.finish();
}

criterion_group!(
    benches,
    grid_creation_benchmark,
    grid_evolution_benchmark,
    glider_simulation_benchmark,
    battle_simulation_benchmark,
    parallel_grid_evolution_benchmark
);
criterion_main!(benches);
