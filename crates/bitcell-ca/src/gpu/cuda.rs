//! CUDA-accelerated CA evolution (NVIDIA GPUs)

use crate::grid::{Grid, Cell, GridSize, GRID_SIZE, LARGE_GRID_SIZE};
use crate::gpu::{GpuEvolver, GpuError, GpuDeviceInfo, GpuBackend};

#[cfg(feature = "cuda")]
use cudarc::{
    driver::{CudaDevice, LaunchAsync, LaunchConfig, CudaSlice},
    nvrtc::Ptx,
};

/// CUDA kernel source for CA evolution
const CUDA_KERNEL: &str = r#"
extern "C" __global__ void evolve_ca(
    const unsigned char* src,
    unsigned char* dst,
    unsigned int size
) {
    const unsigned int x = blockIdx.x * blockDim.x + threadIdx.x;
    const unsigned int y = blockIdx.y * blockDim.y + threadIdx.y;
    
    if (x >= size || y >= size) {
        return;
    }
    
    const unsigned int idx = y * size + x;
    const unsigned char cell = src[idx];
    
    // Count live neighbors and sum their energy
    int live_count = 0;
    unsigned int energy_sum = 0;
    
    // Moore neighborhood (8 neighbors with toroidal wrapping)
    for (int dy = -1; dy <= 1; dy++) {
        for (int dx = -1; dx <= 1; dx++) {
            if (dx == 0 && dy == 0) continue;
            
            // Toroidal wrap
            unsigned int nx = (x + dx + size) % size;
            unsigned int ny = (y + dy + size) % size;
            unsigned int nidx = ny * size + nx;
            
            unsigned char neighbor = src[nidx];
            if (neighbor > 0) {
                live_count++;
                energy_sum += neighbor;
            }
        }
    }
    
    // Apply Conway rules with energy
    unsigned char new_cell;
    if (cell > 0) {
        // Cell is alive
        if (live_count == 2 || live_count == 3) {
            // Survival - keep current energy
            new_cell = cell;
        } else {
            // Death
            new_cell = 0;
        }
    } else {
        // Cell is dead
        if (live_count == 3) {
            // Birth - average energy of neighbors
            unsigned char avg_energy = (unsigned char)(energy_sum / live_count);
            new_cell = max((unsigned char)1, avg_energy);
        } else {
            // Stay dead
            new_cell = 0;
        }
    }
    
    dst[idx] = new_cell;
}
"#;

/// CUDA-based CA evolver
#[cfg(feature = "cuda")]
pub struct CudaEvolver {
    device_info: GpuDeviceInfo,
    device: CudaDevice,
}

#[cfg(feature = "cuda")]
impl CudaEvolver {
    /// Create a new CUDA evolver
    pub fn new() -> Result<Self, GpuError> {
        // Initialize CUDA device
        let device = CudaDevice::new(0)
            .map_err(|e| GpuError::InitializationFailed(format!("Failed to initialize CUDA device: {}", e)))?;
        
        let device_name = device.name()
            .map_err(|e| GpuError::InitializationFailed(format!("Failed to get device name: {}", e)))?;
        
        let device_memory = device.total_memory()
            .map_err(|e| GpuError::InitializationFailed(format!("Failed to get device memory: {}", e)))?;
        
        // Get compute units (multiprocessors)
        let compute_units = device.attribute(cudarc::driver::sys::CUdevice_attribute::CU_DEVICE_ATTRIBUTE_MULTIPROCESSOR_COUNT)
            .map_err(|e| GpuError::InitializationFailed(format!("Failed to get compute units: {}", e)))?;
        
        // Load and compile the kernel
        device.load_ptx(
            Ptx::from_src(CUDA_KERNEL),
            "evolve_ca",
            &["evolve_ca"]
        ).map_err(|e| GpuError::InitializationFailed(format!("Failed to load CUDA kernel: {}", e)))?;
        
        let device_info = GpuDeviceInfo {
            backend: GpuBackend::Cuda,
            name: device_name,
            memory: device_memory,
            compute_units: compute_units as usize,
        };
        
        Ok(Self {
            device_info,
            device,
        })
    }
}

#[cfg(feature = "cuda")]
impl GpuEvolver for CudaEvolver {
    fn evolve(&self, src: &Grid) -> Result<Grid, GpuError> {
        let mut dst = if src.grid_size() == LARGE_GRID_SIZE {
            Grid::with_size(GridSize::Large)
        } else {
            Grid::new()
        };
        self.evolve_into(src, &mut dst)?;
        Ok(dst)
    }
    
    fn evolve_into(&self, src: &Grid, dst: &mut Grid) -> Result<(), GpuError> {
        let size = src.grid_size();
        let num_cells = size * size;
        
        // Ensure dst matches src size
        if dst.cells.len() != num_cells || dst.size != size {
            return Err(GpuError::UnsupportedGridSize(size));
        }
        
        // Extract raw cell data
        let src_data: Vec<u8> = src.cells.iter().map(|c| c.state).collect();
        
        // Allocate GPU memory
        let src_gpu = self.device.htod_copy(src_data)
            .map_err(|e| GpuError::MemoryTransferFailed(format!("Upload failed: {}", e)))?;
        
        let mut dst_gpu = self.device.alloc_zeros::<u8>(num_cells)
            .map_err(|_| GpuError::MemoryAllocationFailed)?;
        
        // Configure kernel launch
        let block_size = 16; // 16x16 threads per block
        let grid_x = (size + block_size - 1) / block_size;
        let grid_y = (size + block_size - 1) / block_size;
        
        let cfg = LaunchConfig {
            grid_dim: (grid_x as u32, grid_y as u32, 1),
            block_dim: (block_size as u32, block_size as u32, 1),
            shared_mem_bytes: 0,
        };
        
        // Launch kernel
        let func = self.device.get_func("evolve_ca", "evolve_ca")
            .map_err(|e| GpuError::KernelExecutionFailed(format!("Failed to get kernel function: {}", e)))?;
        
        unsafe {
            func.launch(cfg, (&src_gpu, &mut dst_gpu, size as u32))
                .map_err(|e| GpuError::KernelExecutionFailed(format!("Kernel launch failed: {}", e)))?;
        }
        
        // Download result from GPU
        let dst_data = self.device.dtoh_sync_copy(&dst_gpu)
            .map_err(|e| GpuError::MemoryTransferFailed(format!("Download failed: {}", e)))?;
        
        // Update dst grid
        for (i, &state) in dst_data.iter().enumerate() {
            dst.cells[i] = Cell { state };
        }
        
        Ok(())
    }
    
    fn device_info(&self) -> &GpuDeviceInfo {
        &self.device_info
    }
}

/// Check if CUDA is available
#[cfg(feature = "cuda")]
pub fn is_available() -> bool {
    CudaDevice::new(0).is_ok()
}

#[cfg(not(feature = "cuda"))]
pub fn is_available() -> bool {
    false
}

#[cfg(test)]
#[cfg(feature = "cuda")]
mod tests {
    use super::*;
    use crate::{Position, Cell};
    
    #[test]
    fn test_cuda_availability() {
        // Just test that the function doesn't panic
        let _ = is_available();
    }
    
    #[test]
    fn test_cuda_evolver_creation() {
        // Try to create an evolver - may fail if no CUDA GPU is available
        let result = CudaEvolver::new();
        // Don't assert success since CI may not have CUDA GPU
        if let Ok(evolver) = result {
            assert!(evolver.device_info().memory > 0);
        }
    }
}
