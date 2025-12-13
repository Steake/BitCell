# BitCell CA - Cellular Automaton Engine

High-performance cellular automaton engine for the BitCell blockchain with GPU acceleration.

## Features

- **Conway's Game of Life with Energy**: Classic cellular automaton rules extended with 8-bit energy values
- **Toroidal Grid**: Seamless wrapping at boundaries
- **Multiple Grid Sizes**: 
  - Standard: 1024×1024 cells
  - Large: 4096×4096 cells
- **GPU Acceleration**: 10x+ speedup using CUDA or OpenCL
- **CPU Fallback**: Automatic fallback with parallel Rayon execution
- **Battle Simulation**: Deterministic CA-based combat for blockchain tournaments

## Quick Start

### Basic Usage

```rust
use bitcell_ca::{Grid, Position, Cell};
use bitcell_ca::rules::evolve_grid;

// Create a grid
let mut grid = Grid::new();

// Add some cells
grid.set(Position::new(100, 100), Cell::alive(128));
grid.set(Position::new(101, 100), Cell::alive(128));
grid.set(Position::new(102, 100), Cell::alive(128));

// Evolve one step
let next_grid = evolve_grid(&grid);
```

### GPU Acceleration

```rust
use bitcell_ca::{Grid, detect_gpu, create_gpu_evolver};

// Detect GPU
match detect_gpu() {
    Some(backend) => println!("GPU available: {:?}", backend),
    None => println!("No GPU - using CPU fallback"),
}

// Create GPU evolver
if let Ok(evolver) = create_gpu_evolver() {
    let grid = Grid::new();
    let next_grid = evolver.evolve(&grid).unwrap();
}
```

### Large Grids

```rust
use bitcell_ca::{Grid, GridSize};

// Create a 4096×4096 grid
let large_grid = Grid::with_size(GridSize::Large);
```

## Building

### Default (CPU only)

```bash
cargo build --package bitcell-ca
cargo test --package bitcell-ca
```

### With OpenCL Support

```bash
cargo build --package bitcell-ca --features opencl
cargo test --package bitcell-ca --features opencl
```

### With CUDA Support

```bash
cargo build --package bitcell-ca --features cuda
cargo test --package bitcell-ca --features cuda
```

## Benchmarking

```bash
# Run all benchmarks
cargo bench --package bitcell-ca

# With GPU support
cargo bench --package bitcell-ca --features opencl
```

## Performance

Expected performance on modern hardware (Intel i7-9700K CPU, NVIDIA RTX 3070):

| Grid Size | CPU (Rayon) | GPU | Speedup |
|-----------|-------------|-----|---------|
| 1024×1024 | ~50 ms      | ~3-5 ms | 10-16x |
| 4096×4096 | ~800 ms     | ~45-60 ms | 13-17x |

## Architecture

### Grid Layout

Cells are stored in a flat `Vec<Cell>` in row-major order:
- Index: `y * grid_size + x`
- Memory: `grid_size × grid_size × 1 byte`
- Standard grid: 1 MB
- Large grid: 16 MB

### Evolution Algorithm

**Conway Rules with Energy**:
1. **Survival**: Live cells with 2-3 neighbors survive (keep energy)
2. **Death**: Live cells with <2 or >3 neighbors die
3. **Birth**: Dead cells with exactly 3 neighbors become alive
4. **Energy Inheritance**: New cells get average energy from neighbors

### GPU Implementation

**CUDA Kernel** (NVIDIA GPUs):
- Block size: 16×16 threads
- Global memory access
- Toroidal wrapping in kernel

**OpenCL Kernel** (AMD/Intel/NVIDIA):
- Work-group size: Runtime determined
- Cross-platform compatibility
- Same algorithm as CUDA

### CPU Fallback

**Rayon Parallel Iterator**:
- Row-level parallelism
- Work-stealing scheduler
- Same results as GPU (bit-exact)

## API Reference

### Core Types

- `Grid`: The cellular automaton grid
- `Cell`: Individual cell with 8-bit energy state
- `Position`: 2D position on the grid
- `GridSize`: Enum for standard/large grids

### Evolution Functions

- `evolve_grid(grid: &Grid) -> Grid`: Evolve one step (CPU)
- `evolve_grid_into(src: &Grid, dst: &mut Grid)`: In-place evolution (CPU)
- `evolve_n_steps(grid: &Grid, steps: usize) -> Grid`: Multi-step evolution

### GPU Functions

- `detect_gpu() -> Option<GpuBackend>`: Detect available GPU
- `create_gpu_evolver() -> Result<Box<dyn GpuEvolver>>`: Create GPU evolver
- `GpuEvolver::evolve(&self, grid: &Grid) -> Result<Grid>`: GPU evolution

## Testing

The test suite includes:
- Unit tests for all grid operations
- Conway rule verification
- GPU vs CPU equivalence tests
- Large grid support tests
- Toroidal wrapping tests

Run tests:
```bash
cargo test --package bitcell-ca --features opencl
```

## Documentation

See [GPU_ACCELERATION.md](./GPU_ACCELERATION.md) for detailed GPU documentation including:
- Performance tuning
- Troubleshooting
- Implementation details
- Future enhancements

## Dependencies

**Core**:
- `serde`: Serialization
- `thiserror`: Error handling
- `rayon`: Parallel CPU execution

**GPU (Optional)**:
- `cudarc`: CUDA support (feature: `cuda`)
- `opencl3`: OpenCL support (feature: `opencl`)

**Testing**:
- `proptest`: Property-based testing
- `criterion`: Benchmarking

## License

See [LICENSE](../../LICENSE) in the repository root.
