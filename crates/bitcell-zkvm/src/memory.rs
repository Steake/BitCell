//! ZKVM Memory Model
//!
//! Simple flat memory model with bounds checking.

use std::collections::HashMap;

/// Memory with sparse storage for efficiency
#[derive(Debug, Clone)]
pub struct Memory {
    data: HashMap<u32, u64>,
    max_address: u32,
}

impl Memory {
    /// Create new memory with maximum addressable space
    pub fn new(max_address: u32) -> Self {
        Self {
            data: HashMap::new(),
            max_address,
        }
    }
    
    /// Load value from memory address
    pub fn load(&self, address: u32) -> Result<u64, String> {
        if address >= self.max_address {
            return Err(format!("Memory access out of bounds: {}", address));
        }
        Ok(*self.data.get(&address).unwrap_or(&0))
    }
    
    /// Store value to memory address
    pub fn store(&mut self, address: u32, value: u64) -> Result<(), String> {
        if address >= self.max_address {
            return Err(format!("Memory access out of bounds: {}", address));
        }
        self.data.insert(address, value);
        Ok(())
    }
    
    /// Get memory size (number of allocated cells)
    pub fn size(&self) -> usize {
        self.data.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_memory_load_store() {
        let mut mem = Memory::new(1000);
        
        mem.store(100, 42).expect("store failed");
        assert_eq!(mem.load(100).expect("load failed"), 42);
        
        // Uninitialized memory returns 0
        assert_eq!(mem.load(200).expect("load failed"), 0);
    }

    #[test]
    fn test_memory_bounds() {
        let mut mem = Memory::new(100);
        
        // Out of bounds access should fail
        assert!(mem.store(200, 42).is_err());
        assert!(mem.load(200).is_err());
    }

    #[test]
    fn test_sparse_memory() {
        let mut mem = Memory::new(1000000);
        
        mem.store(0, 1).unwrap();
        mem.store(999999, 2).unwrap();
        
        // Only 2 cells should be allocated
        assert_eq!(mem.size(), 2);
    }
}
