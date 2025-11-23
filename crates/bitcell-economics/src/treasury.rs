//! Treasury Management

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Treasury for protocol development and grants
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Treasury {
    balance: u64,
    allocations: HashMap<String, u64>,
}

impl Treasury {
    pub fn new() -> Self {
        Self {
            balance: 0,
            allocations: HashMap::new(),
        }
    }
    
    /// Add funds to treasury
    pub fn deposit(&mut self, amount: u64) {
        self.balance += amount;
    }
    
    /// Allocate funds for a purpose
    pub fn allocate(&mut self, purpose: String, amount: u64) -> Result<(), String> {
        if amount > self.balance {
            return Err("Insufficient treasury balance".to_string());
        }
        
        self.balance -= amount;
        *self.allocations.entry(purpose).or_insert(0) += amount;
        Ok(())
    }
    
    /// Get current balance
    pub fn balance(&self) -> u64 {
        self.balance
    }
    
    /// Get allocation for a purpose
    pub fn allocation(&self, purpose: &str) -> u64 {
        *self.allocations.get(purpose).unwrap_or(&0)
    }
    
    /// Get all allocations
    pub fn allocations(&self) -> &HashMap<String, u64> {
        &self.allocations
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_treasury_deposit() {
        let mut treasury = Treasury::new();
        
        treasury.deposit(1000);
        assert_eq!(treasury.balance(), 1000);
        
        treasury.deposit(500);
        assert_eq!(treasury.balance(), 1500);
    }

    #[test]
    fn test_treasury_allocation() {
        let mut treasury = Treasury::new();
        treasury.deposit(1000);
        
        treasury.allocate("Development".to_string(), 300).unwrap();
        assert_eq!(treasury.balance(), 700);
        assert_eq!(treasury.allocation("Development"), 300);
    }

    #[test]
    fn test_insufficient_balance() {
        let mut treasury = Treasury::new();
        treasury.deposit(100);
        
        let result = treasury.allocate("Grant".to_string(), 200);
        assert!(result.is_err());
        assert_eq!(treasury.balance(), 100);
    }

    #[test]
    fn test_multiple_allocations() {
        let mut treasury = Treasury::new();
        treasury.deposit(1000);
        
        treasury.allocate("Dev".to_string(), 300).unwrap();
        treasury.allocate("Marketing".to_string(), 200).unwrap();
        treasury.allocate("Dev".to_string(), 100).unwrap();
        
        assert_eq!(treasury.balance(), 400);
        assert_eq!(treasury.allocation("Dev"), 400);
        assert_eq!(treasury.allocation("Marketing"), 200);
    }
}
