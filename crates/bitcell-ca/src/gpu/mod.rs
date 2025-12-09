//! GPU-accelerated CA evolution
//!
//! This module provides GPU acceleration for cellular automaton evolution
//! using CUDA (NVIDIA) and OpenCL (AMD/Intel) backends with automatic
//! fallback to CPU when GPU is not available.

use crate::grid::{Grid, Cell, Position};

#[cfg(feature = "cuda")]
pub mod cuda;

#[cfg(feature = "opencl")]
pub mod opencl;

/// GPU backend type for CA evolution
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GpuBackend {
    /// NVIDIA CUDA backend
    #[cfg(feature = "cuda")]
    Cuda,
    /// OpenCL backend (AMD/Intel/NVIDIA)
    #[cfg(feature = "opencl")]
    OpenCL,
    /// CPU fallback (no GPU)
    Cpu,
}

/// GPU device information
#[derive(Debug, Clone)]
pub struct GpuDeviceInfo {
    pub backend: GpuBackend,
    pub name: String,
    pub memory: usize, // in bytes
    pub compute_units: usize,
}

/// Trait for GPU-accelerated CA evolution
pub trait GpuEvolver: Send + Sync {
    /// Evolve a grid one step using GPU acceleration
    fn evolve(&self, src: &Grid) -> Result<Grid, GpuError>;
    
    /// Evolve a grid one step in-place using GPU acceleration
    fn evolve_into(&self, src: &Grid, dst: &mut Grid) -> Result<(), GpuError>;
    
    /// Get device information
    fn device_info(&self) -> &GpuDeviceInfo;
}

/// GPU-related errors
#[derive(Debug, thiserror::Error)]
pub enum GpuError {
    #[error("GPU not available")]
    NotAvailable,
    
    #[error("GPU initialization failed: {0}")]
    InitializationFailed(String),
    
    #[error("GPU memory allocation failed")]
    MemoryAllocationFailed,
    
    #[error("GPU kernel execution failed: {0}")]
    KernelExecutionFailed(String),
    
    #[error("GPU memory transfer failed: {0}")]
    MemoryTransferFailed(String),
    
    #[error("Unsupported grid size: {0}")]
    UnsupportedGridSize(usize),
}

/// Detect available GPU devices
pub fn detect_gpu() -> Option<GpuBackend> {
    #[cfg(feature = "cuda")]
    {
        if cuda::is_available() {
            return Some(GpuBackend::Cuda);
        }
    }
    
    #[cfg(feature = "opencl")]
    {
        if opencl::is_available() {
            return Some(GpuBackend::OpenCL);
        }
    }
    
    None
}

/// Create a GPU evolver with automatic backend selection
pub fn create_gpu_evolver() -> Result<Box<dyn GpuEvolver>, GpuError> {
    #[cfg(feature = "cuda")]
    {
        if let Ok(evolver) = cuda::CudaEvolver::new() {
            return Ok(Box::new(evolver));
        }
    }
    
    #[cfg(feature = "opencl")]
    {
        if let Ok(evolver) = opencl::OpenCLEvolver::new() {
            return Ok(Box::new(evolver));
        }
    }
    
    Err(GpuError::NotAvailable)
}

/// Create a GPU evolver with specific backend
pub fn create_gpu_evolver_with_backend(backend: GpuBackend) -> Result<Box<dyn GpuEvolver>, GpuError> {
    match backend {
        #[cfg(feature = "cuda")]
        GpuBackend::Cuda => {
            Ok(Box::new(cuda::CudaEvolver::new()?))
        }
        #[cfg(feature = "opencl")]
        GpuBackend::OpenCL => {
            Ok(Box::new(opencl::OpenCLEvolver::new()?))
        }
        GpuBackend::Cpu => {
            Err(GpuError::NotAvailable)
        }
        #[allow(unreachable_patterns)]
        _ => Err(GpuError::NotAvailable),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gpu_detection() {
        // This test just checks that detection doesn't panic
        let _backend = detect_gpu();
    }
}
