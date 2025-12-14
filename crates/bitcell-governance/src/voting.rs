//! Voting mechanisms and delegation

use serde::{Deserialize, Serialize};

/// Vote type
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum VoteType {
    /// Vote in favor
    For,
    
    /// Vote against
    Against,
    
    /// Abstain from voting (counted for quorum but not for/against)
    Abstain,
}

/// Voting power calculation
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct VotingPower {
    /// Raw token amount
    pub tokens: u64,
    
    /// Effective voting power (after quadratic calculation if applicable)
    pub value: u64,
}

impl VotingPower {
    /// Linear voting: 1 CELL = 1 vote
    pub fn linear(tokens: u64) -> Self {
        Self {
            tokens,
            value: tokens,
        }
    }
    
    /// Quadratic voting: voting power = sqrt(tokens)
    /// This helps prevent plutocracy by reducing the power of large token holders
    pub fn quadratic(tokens: u64) -> Self {
        let value = integer_sqrt(tokens);
        Self {
            tokens,
            value,
        }
    }
}

/// Integer square root using Newton's method
/// Uses checked arithmetic to prevent overflow
fn integer_sqrt(n: u64) -> u64 {
    if n == 0 {
        return 0;
    }
    if n == 1 {
        return 1;
    }
    
    let mut x = n;
    let mut y = (x + 1) / 2;
    
    while y < x {
        x = y;
        // Use checked division to prevent overflow
        if let Some(div) = n.checked_div(x) {
            y = (x + div) / 2;
        } else {
            break;
        }
    }
    
    x
}

/// A vote cast on a proposal
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Vote {
    /// Voter's public key
    #[serde(with = "crate::serde_pubkey")]
    pub voter: [u8; 33],
    
    /// Type of vote
    pub vote_type: VoteType,
    
    /// Voting power used
    pub voting_power: VotingPower,
    
    /// Block number when vote was cast
    pub block_number: u64,
}

/// Delegation of voting power
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Delegation {
    /// Delegator's public key
    #[serde(with = "crate::serde_pubkey")]
    pub delegator: [u8; 33],
    
    /// Delegatee's public key (who receives the voting power)
    #[serde(with = "crate::serde_pubkey")]
    pub delegatee: [u8; 33],
    
    /// Amount of voting power delegated
    pub amount: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_linear_voting_power() {
        let power = VotingPower::linear(1000);
        assert_eq!(power.tokens, 1000);
        assert_eq!(power.value, 1000);
    }
    
    #[test]
    fn test_quadratic_voting_power() {
        let power = VotingPower::quadratic(10000);
        assert_eq!(power.tokens, 10000);
        assert_eq!(power.value, 100); // sqrt(10000) = 100
    }
    
    #[test]
    fn test_integer_sqrt() {
        assert_eq!(integer_sqrt(0), 0);
        assert_eq!(integer_sqrt(1), 1);
        assert_eq!(integer_sqrt(4), 2);
        assert_eq!(integer_sqrt(9), 3);
        assert_eq!(integer_sqrt(16), 4);
        assert_eq!(integer_sqrt(100), 10);
        assert_eq!(integer_sqrt(10000), 100);
    }
    
    #[test]
    fn test_vote_creation() {
        let vote = Vote {
            voter: [1u8; 33],
            vote_type: VoteType::For,
            voting_power: VotingPower::linear(5000),
            block_number: 100,
        };
        
        assert_eq!(vote.voting_power.value, 5000);
    }
    
    #[test]
    fn test_delegation() {
        let delegation = Delegation {
            delegator: [1u8; 33],
            delegatee: [2u8; 33],
            amount: 1000,
        };
        
        assert_eq!(delegation.amount, 1000);
    }
}
