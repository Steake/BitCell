//! Timelock configuration for different proposal types

use serde::{Deserialize, Serialize};
use crate::proposal::ProposalType;

/// Timelock configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimelockConfig {
    /// Timelock for parameter changes (in seconds)
    pub parameter_change_delay: u64,
    
    /// Timelock for treasury spending (in seconds)
    pub treasury_spending_delay: u64,
    
    /// Timelock for protocol upgrades (in seconds)
    pub protocol_upgrade_delay: u64,
}

impl Default for TimelockConfig {
    fn default() -> Self {
        Self {
            parameter_change_delay: 2 * 24 * 60 * 60, // 2 days
            treasury_spending_delay: 6 * 60 * 60,      // 6 hours
            protocol_upgrade_delay: 2 * 24 * 60 * 60,  // 2 days
        }
    }
}

impl TimelockConfig {
    /// Get the timelock duration for a proposal type
    pub fn get_duration(&self, proposal_type: &ProposalType) -> u64 {
        match proposal_type {
            ProposalType::ParameterChange { .. } => self.parameter_change_delay,
            ProposalType::TreasurySpending { .. } => self.treasury_spending_delay,
            ProposalType::ProtocolUpgrade { .. } => self.protocol_upgrade_delay,
        }
    }
}

/// Timelock state for a proposal
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Timelock {
    /// When the timelock started
    pub start_time: u64,
    
    /// Duration of the timelock (in seconds)
    pub duration: u64,
}

impl Timelock {
    /// Create a new timelock
    pub fn new(start_time: u64, duration: u64) -> Self {
        Self {
            start_time,
            duration,
        }
    }
    
    /// Check if the timelock has expired
    pub fn is_expired(&self, current_time: u64) -> bool {
        current_time >= self.expiry_time()
    }
    
    /// Get the expiry time
    pub fn expiry_time(&self) -> u64 {
        self.start_time.saturating_add(self.duration)
    }
    
    /// Get remaining time (0 if expired)
    pub fn remaining_time(&self, current_time: u64) -> u64 {
        let expiry = self.expiry_time();
        if current_time >= expiry {
            0
        } else {
            expiry.saturating_sub(current_time)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_timelock_config_default() {
        let config = TimelockConfig::default();
        
        assert_eq!(config.parameter_change_delay, 2 * 24 * 60 * 60);
        assert_eq!(config.treasury_spending_delay, 6 * 60 * 60);
        assert_eq!(config.protocol_upgrade_delay, 2 * 24 * 60 * 60);
    }
    
    #[test]
    fn test_timelock_duration() {
        let config = TimelockConfig::default();
        
        let param_type = ProposalType::ParameterChange {
            parameter: "test".to_string(),
            new_value: "value".to_string(),
        };
        assert_eq!(config.get_duration(&param_type), 2 * 24 * 60 * 60);
        
        let treasury_type = ProposalType::TreasurySpending {
            recipient: [0u8; 33],
            amount: 1000,
            reason: "test".to_string(),
        };
        assert_eq!(config.get_duration(&treasury_type), 6 * 60 * 60);
        
        let upgrade_type = ProposalType::ProtocolUpgrade {
            version: "1.0.0".to_string(),
            code_hash: [0u8; 32],
            description: "test".to_string(),
        };
        assert_eq!(config.get_duration(&upgrade_type), 2 * 24 * 60 * 60);
    }
    
    #[test]
    fn test_timelock_expiry() {
        let timelock = Timelock::new(1000, 3600); // 1 hour delay
        
        assert_eq!(timelock.expiry_time(), 4600);
        assert!(!timelock.is_expired(1000));
        assert!(!timelock.is_expired(4599));
        assert!(timelock.is_expired(4600));
        assert!(timelock.is_expired(5000));
    }
    
    #[test]
    fn test_remaining_time() {
        let timelock = Timelock::new(1000, 3600);
        
        assert_eq!(timelock.remaining_time(1000), 3600);
        assert_eq!(timelock.remaining_time(2000), 2600);
        assert_eq!(timelock.remaining_time(4600), 0);
        assert_eq!(timelock.remaining_time(5000), 0);
    }
}
