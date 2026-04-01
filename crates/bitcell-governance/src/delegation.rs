//! Vote delegation system

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A delegation record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Delegation {
    /// Address delegating their voting power
    pub delegator: [u8; 33],
    
    /// Address receiving the delegated voting power
    pub delegatee: [u8; 33],
    
    /// Amount of voting power delegated
    pub amount: u64,
    
    /// Timestamp when delegation was created
    pub created_at: u64,
}

/// Manages vote delegations
pub struct DelegationManager {
    /// Active delegations: delegator -> (delegatee -> amount)
    delegations: HashMap<[u8; 33], HashMap<[u8; 33], u64>>,
    
    /// Reverse index: delegatee -> total delegated power
    delegated_power: HashMap<[u8; 33], u64>,
}

impl DelegationManager {
    pub fn new() -> Self {
        Self {
            delegations: HashMap::new(),
            delegated_power: HashMap::new(),
        }
    }
    
    /// Delegate voting power to another address
    pub fn delegate(
        &mut self,
        delegator: [u8; 33],
        delegatee: [u8; 33],
        amount: u64,
    ) -> crate::Result<()> {
        // Prevent self-delegation
        if delegator == delegatee {
            return Err(crate::Error::InvalidDelegation);
        }
        
        // Update delegations map
        let delegator_map = self.delegations.entry(delegator).or_insert_with(HashMap::new);
        
        // If already delegating to this address, add to existing amount
        let current = delegator_map.get(&delegatee).copied().unwrap_or(0);
        let new_amount = current.saturating_add(amount);
        delegator_map.insert(delegatee, new_amount);
        
        // Update delegated power index
        let total = self.delegated_power.entry(delegatee).or_insert(0);
        *total = total.saturating_add(amount);
        
        tracing::info!(
            delegator = %hex::encode(&delegator),
            delegatee = %hex::encode(&delegatee),
            amount = amount,
            "Voting power delegated"
        );
        
        Ok(())
    }
    
    /// Remove a delegation
    pub fn undelegate(
        &mut self,
        delegator: [u8; 33],
        delegatee: [u8; 33],
    ) -> crate::Result<()> {
        // Get delegation amount
        let amount = self.delegations
            .get(&delegator)
            .and_then(|m| m.get(&delegatee))
            .copied()
            .ok_or(crate::Error::InvalidDelegation)?;
        
        // Remove from delegations map
        if let Some(delegator_map) = self.delegations.get_mut(&delegator) {
            delegator_map.remove(&delegatee);
            
            // Clean up empty maps
            if delegator_map.is_empty() {
                self.delegations.remove(&delegator);
            }
        }
        
        // Update delegated power index
        if let Some(total) = self.delegated_power.get_mut(&delegatee) {
            *total = total.saturating_sub(amount);
            
            // Clean up zero entries
            if *total == 0 {
                self.delegated_power.remove(&delegatee);
            }
        }
        
        tracing::info!(
            delegator = %hex::encode(&delegator),
            delegatee = %hex::encode(&delegatee),
            amount = amount,
            "Delegation removed"
        );
        
        Ok(())
    }
    
    /// Get total voting power delegated to an address
    pub fn get_delegated_power(&self, delegatee: &[u8; 33]) -> u64 {
        self.delegated_power.get(delegatee).copied().unwrap_or(0)
    }
    
    /// Get all delegations from an address
    pub fn get_delegations(&self, delegator: &[u8; 33]) -> Vec<([u8; 33], u64)> {
        self.delegations
            .get(delegator)
            .map(|m| m.iter().map(|(k, v)| (*k, *v)).collect())
            .unwrap_or_default()
    }
}

impl Default for DelegationManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_delegation() {
        let mut manager = DelegationManager::new();
        let delegator = [1u8; 33];
        let delegatee = [2u8; 33];
        
        // Delegate 1000 power
        manager.delegate(delegator, delegatee, 1000).unwrap();
        
        // Check delegated power
        assert_eq!(manager.get_delegated_power(&delegatee), 1000);
        
        // Check delegations
        let delegations = manager.get_delegations(&delegator);
        assert_eq!(delegations.len(), 1);
        assert_eq!(delegations[0], (delegatee, 1000));
    }
    
    #[test]
    fn test_multiple_delegations() {
        let mut manager = DelegationManager::new();
        let delegator = [1u8; 33];
        let delegatee1 = [2u8; 33];
        let delegatee2 = [3u8; 33];
        
        // Delegate to two different addresses
        manager.delegate(delegator, delegatee1, 500).unwrap();
        manager.delegate(delegator, delegatee2, 300).unwrap();
        
        // Check delegated power
        assert_eq!(manager.get_delegated_power(&delegatee1), 500);
        assert_eq!(manager.get_delegated_power(&delegatee2), 300);
        
        // Check delegations
        let delegations = manager.get_delegations(&delegator);
        assert_eq!(delegations.len(), 2);
    }
    
    #[test]
    fn test_undelegate() {
        let mut manager = DelegationManager::new();
        let delegator = [1u8; 33];
        let delegatee = [2u8; 33];
        
        // Delegate and then undelegate
        manager.delegate(delegator, delegatee, 1000).unwrap();
        assert_eq!(manager.get_delegated_power(&delegatee), 1000);
        
        manager.undelegate(delegator, delegatee).unwrap();
        assert_eq!(manager.get_delegated_power(&delegatee), 0);
        
        // Should be empty
        let delegations = manager.get_delegations(&delegator);
        assert_eq!(delegations.len(), 0);
    }
    
    #[test]
    fn test_self_delegation_prevented() {
        let mut manager = DelegationManager::new();
        let address = [1u8; 33];
        
        // Self-delegation should fail
        let result = manager.delegate(address, address, 1000);
        assert!(matches!(result, Err(crate::Error::InvalidDelegation)));
    }
    
    #[test]
    fn test_accumulated_delegation() {
        let mut manager = DelegationManager::new();
        let delegator = [1u8; 33];
        let delegatee = [2u8; 33];
        
        // Multiple delegations to same address accumulate
        manager.delegate(delegator, delegatee, 100).unwrap();
        manager.delegate(delegator, delegatee, 200).unwrap();
        
        assert_eq!(manager.get_delegated_power(&delegatee), 300);
    }
    
    #[test]
    fn test_multiple_delegators() {
        let mut manager = DelegationManager::new();
        let delegator1 = [1u8; 33];
        let delegator2 = [2u8; 33];
        let delegatee = [3u8; 33];
        
        // Two different delegators delegate to same address
        manager.delegate(delegator1, delegatee, 500).unwrap();
        manager.delegate(delegator2, delegatee, 300).unwrap();
        
        // Total delegated power should be sum
        assert_eq!(manager.get_delegated_power(&delegatee), 800);
    }
}
