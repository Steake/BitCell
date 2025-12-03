//! System metrics collection
//!
//! Provides real-time system metrics including CPU, memory, disk, and uptime.
//! Uses the sysinfo crate for cross-platform system information.

use sysinfo::{System, Disks, CpuRefreshKind, MemoryRefreshKind, RefreshKind};
use std::sync::RwLock;
use std::time::Instant;

/// System metrics data
#[derive(Debug, Clone)]
pub struct SystemMetrics {
    /// Node uptime in seconds
    pub uptime_seconds: u64,
    /// CPU usage percentage (0.0 - 100.0)
    pub cpu_usage: f64,
    /// Memory usage in megabytes
    pub memory_usage_mb: u64,
    /// Total memory in megabytes
    pub total_memory_mb: u64,
    /// Disk usage in megabytes
    pub disk_usage_mb: u64,
    /// Total disk space in megabytes
    pub total_disk_mb: u64,
}

impl Default for SystemMetrics {
    fn default() -> Self {
        Self {
            uptime_seconds: 0,
            cpu_usage: 0.0,
            memory_usage_mb: 0,
            total_memory_mb: 0,
            disk_usage_mb: 0,
            total_disk_mb: 0,
        }
    }
}

/// System metrics collector
/// 
/// Collects real-time system metrics including:
/// - CPU usage (average across all cores)
/// - Memory usage
/// - Disk usage
/// - Process uptime
pub struct SystemMetricsCollector {
    system: RwLock<System>,
    disks: RwLock<Disks>,
    start_time: Instant,
}

impl SystemMetricsCollector {
    /// Create a new system metrics collector
    pub fn new() -> Self {
        let refresh_kind = RefreshKind::new()
            .with_cpu(CpuRefreshKind::everything())
            .with_memory(MemoryRefreshKind::everything());
        
        Self {
            system: RwLock::new(System::new_with_specifics(refresh_kind)),
            disks: RwLock::new(Disks::new_with_refreshed_list()),
            start_time: Instant::now(),
        }
    }
    
    /// Collect current system metrics
    /// 
    /// This refreshes system information and returns current metrics.
    /// Call this periodically to get updated metrics.
    pub fn collect(&self) -> SystemMetrics {
        // Refresh CPU and memory
        let (cpu_usage, memory_usage_mb, total_memory_mb) = {
            let mut system = self.system.write().unwrap();
            system.refresh_all();
            
            // Calculate average CPU usage across all cores
            let cpu_usage = if system.cpus().is_empty() {
                0.0
            } else {
                system.cpus().iter()
                    .map(|cpu| cpu.cpu_usage() as f64)
                    .sum::<f64>() / system.cpus().len() as f64
            };
            
            // Memory usage in MB
            let memory_usage_mb = system.used_memory() / 1024 / 1024;
            let total_memory_mb = system.total_memory() / 1024 / 1024;
            
            (cpu_usage, memory_usage_mb, total_memory_mb)
        };
        
        // Refresh disk info
        let (disk_usage_mb, total_disk_mb) = {
            let mut disks = self.disks.write().unwrap();
            disks.refresh();
            
            let mut total_used: u64 = 0;
            let mut total_space: u64 = 0;
            
            for disk in disks.iter() {
                total_space += disk.total_space();
                total_used += disk.total_space() - disk.available_space();
            }
            
            (total_used / 1024 / 1024, total_space / 1024 / 1024)
        };
        
        SystemMetrics {
            uptime_seconds: self.start_time.elapsed().as_secs(),
            cpu_usage,
            memory_usage_mb,
            total_memory_mb,
            disk_usage_mb,
            total_disk_mb,
        }
    }
    
    /// Get uptime in seconds
    pub fn uptime(&self) -> u64 {
        self.start_time.elapsed().as_secs()
    }
}

impl Default for SystemMetricsCollector {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_system_metrics_collector_creation() {
        let collector = SystemMetricsCollector::new();
        assert_eq!(collector.uptime(), 0);
    }
    
    #[test]
    fn test_system_metrics_collection() {
        let collector = SystemMetricsCollector::new();
        let metrics = collector.collect();
        
        // CPU usage should be between 0 and 100
        assert!(metrics.cpu_usage >= 0.0 && metrics.cpu_usage <= 100.0);
        
        // Memory should be positive
        assert!(metrics.total_memory_mb > 0);
    }
    
    #[test]
    fn test_uptime_increases() {
        let collector = SystemMetricsCollector::new();
        let initial = collector.uptime();
        
        // Sleep briefly
        std::thread::sleep(std::time::Duration::from_millis(10));
        
        // Uptime should be same or greater (accounting for timing)
        let later = collector.uptime();
        assert!(later >= initial);
    }
}
