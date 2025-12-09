//! OpenCL-accelerated CA evolution (AMD/Intel/NVIDIA GPUs)

use crate::grid::{Grid, Cell, GridSize, GRID_SIZE, LARGE_GRID_SIZE};
use crate::gpu::{GpuEvolver, GpuError, GpuDeviceInfo, GpuBackend};

#[cfg(feature = "opencl")]
use opencl3::{
    device::{Device, CL_DEVICE_TYPE_GPU, get_all_devices},
    context::Context,
    command_queue::{CommandQueue, CL_QUEUE_PROFILING_ENABLE},
    memory::{Buffer, CL_MEM_READ_ONLY, CL_MEM_WRITE_ONLY},
    program::Program,
    kernel::{Kernel, ExecuteKernel},
    types::{cl_uchar, CL_BLOCKING},
};

/// OpenCL kernel source for CA evolution
const OPENCL_KERNEL: &str = r#"
// Conway's Game of Life with energy - OpenCL kernel
__kernel void evolve_ca(
    __global const uchar* src,
    __global uchar* dst,
    const uint size
) {
    const int x = get_global_id(0);
    const int y = get_global_id(1);
    
    if (x >= size || y >= size) {
        return;
    }
    
    const int idx = y * size + x;
    const uchar cell = src[idx];
    
    // Count live neighbors and sum their energy
    int live_count = 0;
    uint energy_sum = 0;
    
    // Moore neighborhood (8 neighbors with toroidal wrapping)
    for (int dy = -1; dy <= 1; dy++) {
        for (int dx = -1; dx <= 1; dx++) {
            if (dx == 0 && dy == 0) continue;
            
            // Toroidal wrap
            int nx = (x + dx + size) % size;
            int ny = (y + dy + size) % size;
            int nidx = ny * size + nx;
            
            uchar neighbor = src[nidx];
            if (neighbor > 0) {
                live_count++;
                energy_sum += neighbor;
            }
        }
    }
    
    // Apply Conway rules with energy
    uchar new_cell;
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
            uchar avg_energy = (uchar)(energy_sum / live_count);
            new_cell = max((uchar)1, avg_energy);
        } else {
            // Stay dead
            new_cell = 0;
        }
    }
    
    dst[idx] = new_cell;
}
"#;

/// OpenCL-based CA evolver
#[cfg(feature = "opencl")]
pub struct OpenCLEvolver {
    device_info: GpuDeviceInfo,
    context: Context,
    queue: CommandQueue,
    program: Program,
}

#[cfg(feature = "opencl")]
impl OpenCLEvolver {
    /// Create a new OpenCL evolver
    pub fn new() -> Result<Self, GpuError> {
        // Find all GPU devices
        let device_ids = get_all_devices(CL_DEVICE_TYPE_GPU)
            .map_err(|e| GpuError::InitializationFailed(format!("Failed to get GPU devices: {:?}", e)))?;
        
        let device_id = device_ids.first()
            .ok_or_else(|| GpuError::NotAvailable)?;
        
        let device = Device::new(*device_id);
        
        let device_name = device.name()
            .map_err(|e| GpuError::InitializationFailed(format!("Failed to get device name: {:?}", e)))?;
        
        let device_memory = device.global_mem_size()
            .map_err(|e| GpuError::InitializationFailed(format!("Failed to get device memory: {:?}", e)))?;
        
        let compute_units = device.max_compute_units()
            .map_err(|e| GpuError::InitializationFailed(format!("Failed to get compute units: {:?}", e)))?;
        
        // Create context and command queue
        let context = Context::from_device(&device)
            .map_err(|e| GpuError::InitializationFailed(format!("Failed to create context: {:?}", e)))?;
        
        let queue = CommandQueue::create_default(&context, CL_QUEUE_PROFILING_ENABLE)
            .map_err(|e| GpuError::InitializationFailed(format!("Failed to create command queue: {:?}", e)))?;
        
        // Build the program
        let program = Program::create_and_build_from_source(&context, OPENCL_KERNEL, "")
            .map_err(|e| GpuError::InitializationFailed(format!("Failed to build OpenCL program: {:?}", e)))?;
        
        let device_info = GpuDeviceInfo {
            backend: GpuBackend::OpenCL,
            name: device_name,
            memory: device_memory as usize,
            compute_units: compute_units as usize,
        };
        
        Ok(Self {
            device_info,
            context,
            queue,
            program,
        })
    }
}

#[cfg(feature = "opencl")]
impl GpuEvolver for OpenCLEvolver {
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
        let src_data: Vec<cl_uchar> = src.cells.iter().map(|c| c.state).collect();
        
        // Create OpenCL buffers
        let mut src_buffer = unsafe {
            Buffer::<cl_uchar>::create(&self.context, CL_MEM_READ_ONLY, num_cells, std::ptr::null_mut())
                .map_err(|_| GpuError::MemoryAllocationFailed)?
        };
        
        let dst_buffer = unsafe {
            Buffer::<cl_uchar>::create(&self.context, CL_MEM_WRITE_ONLY, num_cells, std::ptr::null_mut())
                .map_err(|_| GpuError::MemoryAllocationFailed)?
        };
        
        // Upload src data to GPU
        unsafe {
            self.queue.enqueue_write_buffer(&mut src_buffer, CL_BLOCKING, 0, &src_data, &[])
                .map_err(|e| GpuError::MemoryTransferFailed(format!("Upload failed: {:?}", e)))?;
        }
        
        // Create and execute kernel
        let kernel = Kernel::create(&self.program, "evolve_ca")
            .map_err(|e| GpuError::KernelExecutionFailed(format!("Kernel creation failed: {:?}", e)))?;
        
        let kernel_event = unsafe {
            ExecuteKernel::new(&kernel)
                .set_arg(&src_buffer)
                .set_arg(&dst_buffer)
                .set_arg(&(size as u32))
                .set_global_work_sizes(&[size, size])
                .enqueue_nd_range(&self.queue)
                .map_err(|e| GpuError::KernelExecutionFailed(format!("Kernel execution failed: {:?}", e)))?
        };
        
        kernel_event.wait()
            .map_err(|e| GpuError::KernelExecutionFailed(format!("Kernel wait failed: {:?}", e)))?;
        
        // Download result from GPU
        let mut dst_data = vec![0u8; num_cells];
        unsafe {
            self.queue.enqueue_read_buffer(&dst_buffer, CL_BLOCKING, 0, &mut dst_data, &[])
                .map_err(|e| GpuError::MemoryTransferFailed(format!("Download failed: {:?}", e)))?;
        }
        
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

/// Check if OpenCL is available
#[cfg(feature = "opencl")]
pub fn is_available() -> bool {
    get_all_devices(CL_DEVICE_TYPE_GPU)
        .map(|devices| !devices.is_empty())
        .unwrap_or(false)
}

#[cfg(not(feature = "opencl"))]
pub fn is_available() -> bool {
    false
}

#[cfg(test)]
#[cfg(feature = "opencl")]
mod tests {
    use super::*;
    use crate::{Position, Cell};
    
    #[test]
    fn test_opencl_availability() {
        // Just test that the function doesn't panic
        let _ = is_available();
    }
    
    #[test]
    fn test_opencl_evolver_creation() {
        // Try to create an evolver - may fail if no GPU is available
        let result = OpenCLEvolver::new();
        // Don't assert success since CI may not have GPU
        if let Ok(evolver) = result {
            assert!(evolver.device_info().memory > 0);
        }
    }
}
