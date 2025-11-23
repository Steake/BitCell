use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use bitcell_ca::{Grid, Glider, GliderPattern, Battle, Position, Cell};
use bitcell_ca::rules::evolve_grid;

fn grid_creation_benchmark(c: &mut Criterion) {
    c.bench_function("grid_1024x1024_creation", |b| {
        b.iter(|| Grid::new())
    });
}

fn grid_evolution_benchmark(c: &mut Criterion) {
    let mut grid = Grid::new();
    // Add some initial patterns
    grid.set(Position::new(100, 100), Cell::alive(128));
    grid.set(Position::new(100, 101), Cell::alive(128));
    grid.set(Position::new(101, 100), Cell::alive(128));

    c.bench_function("grid_evolution_step", |b| {
        b.iter(|| {
            let g = grid.clone();
            black_box(evolve_grid(&g))
        });
    });
}

fn glider_creation_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("glider_creation");

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
                let mut grid = Grid::new();
                grid.set_pattern(glider.position, &glider.cells());
                black_box(grid)
            });
        });
    }
    group.finish();
}

fn battle_simulation_benchmark(c: &mut Criterion) {
    c.bench_function("battle_simulation", |b| {
        let glider_a = Glider::new(GliderPattern::Heavyweight, Position::new(200, 200));
        let glider_b = Glider::new(GliderPattern::Standard, Position::new(800, 800));
        let battle = Battle::new(glider_a, glider_b);

        b.iter(|| {
            let b = battle.clone();
            black_box(b.simulate())
        });
    });
}

fn parallel_grid_evolution_benchmark(c: &mut Criterion) {
    let mut grid = Grid::new();
    // Add scattered patterns for realistic parallel workload
    for i in 0..10 {
        for j in 0..10 {
            grid.set(Position::new(i * 100, j * 100), Cell::alive(200));
        }
    }

    c.bench_function("parallel_evolution_step", |b| {
        b.iter(|| {
            let g = grid.clone();
            black_box(evolve_grid(&g))
        });
    });
}

criterion_group!(
    benches,
    grid_creation_benchmark,
    grid_evolution_benchmark,
    glider_creation_benchmark,
    battle_simulation_benchmark,
    parallel_grid_evolution_benchmark
);
criterion_main!(benches);
