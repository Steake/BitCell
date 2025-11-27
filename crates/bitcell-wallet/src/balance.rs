//! Balance Display and Management
//!
//! Provides balance tracking and display formatting for multiple chains.

use crate::{Address, Chain};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Balance for a specific chain/address
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Balance {
    /// Balance in smallest unit (satoshis, wei, cells)
    amount: u64,
    /// Associated chain
    chain: Chain,
}

impl Balance {
    /// Create a new balance
    pub fn new(amount: u64, chain: Chain) -> Self {
        Self { amount, chain }
    }

    /// Create zero balance
    pub fn zero(chain: Chain) -> Self {
        Self { amount: 0, chain }
    }

    /// Get the raw amount in smallest units
    pub fn amount(&self) -> u64 {
        self.amount
    }

    /// Get the associated chain
    pub fn chain(&self) -> Chain {
        self.chain
    }

    /// Format the balance as a human-readable string with proper decimals
    pub fn format(&self) -> String {
        let decimals = self.chain.decimals() as u32;
        let divisor = 10u64.pow(decimals);
        let whole = self.amount / divisor;
        let fraction = self.amount % divisor;
        
        if fraction == 0 {
            format!("{} {}", whole, self.chain.symbol())
        } else {
            // Format with proper decimal places, removing trailing zeros
            let fraction_str = format!("{:0>width$}", fraction, width = decimals as usize);
            let trimmed = fraction_str.trim_end_matches('0');
            format!("{}.{} {}", whole, trimmed, self.chain.symbol())
        }
    }

    /// Format with fixed decimal places
    pub fn format_fixed(&self, decimal_places: u8) -> String {
        let decimals = self.chain.decimals() as u32;
        let divisor = 10u64.pow(decimals);
        let whole = self.amount / divisor;
        let fraction = self.amount % divisor;
        
        // Scale fraction to desired decimal places
        let scale = 10u64.pow(decimal_places as u32);
        let scaled_fraction = (fraction * scale) / divisor;
        
        format!(
            "{}.{:0>width$} {}",
            whole,
            scaled_fraction,
            self.chain.symbol(),
            width = decimal_places as usize
        )
    }

    /// Add to balance (returns new balance, doesn't modify self)
    pub fn add(&self, amount: u64) -> Self {
        Self {
            amount: self.amount.saturating_add(amount),
            chain: self.chain,
        }
    }

    /// Subtract from balance (returns None if insufficient)
    pub fn subtract(&self, amount: u64) -> Option<Self> {
        if amount > self.amount {
            None
        } else {
            Some(Self {
                amount: self.amount - amount,
                chain: self.chain,
            })
        }
    }

    /// Check if balance is sufficient for amount
    pub fn is_sufficient(&self, amount: u64) -> bool {
        self.amount >= amount
    }

    /// Convert amount from one unit to smallest unit
    pub fn from_units(amount: f64, chain: Chain) -> Self {
        let decimals = chain.decimals() as u32;
        let multiplier = 10u64.pow(decimals);
        let smallest = (amount * multiplier as f64) as u64;
        Self::new(smallest, chain)
    }

    /// Convert to standard units (BTC, ETH, etc.)
    pub fn to_units(&self) -> f64 {
        let decimals = self.chain.decimals() as u32;
        let divisor = 10u64.pow(decimals);
        self.amount as f64 / divisor as f64
    }
}

impl Default for Balance {
    fn default() -> Self {
        Self::zero(Chain::BitCell)
    }
}

impl std::fmt::Display for Balance {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.format())
    }
}

/// Balance tracker for multiple addresses
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct BalanceTracker {
    /// Balances by address string
    balances: HashMap<String, Balance>,
    /// Cached total by chain
    #[serde(skip)]
    totals: HashMap<Chain, Balance>,
}

impl BalanceTracker {
    /// Create a new balance tracker
    pub fn new() -> Self {
        Self {
            balances: HashMap::new(),
            totals: HashMap::new(),
        }
    }

    /// Set balance for an address
    pub fn set_balance(&mut self, address: &Address, balance: Balance) {
        let key = address.to_string_formatted();
        self.balances.insert(key, balance);
        self.update_totals();
    }

    /// Get balance for an address
    pub fn get_balance(&self, address: &Address) -> Balance {
        let key = address.to_string_formatted();
        self.balances.get(&key).copied().unwrap_or_else(|| Balance::zero(address.chain()))
    }

    /// Get total balance for a chain
    pub fn get_total(&self, chain: Chain) -> Balance {
        self.totals.get(&chain).copied().unwrap_or_else(|| Balance::zero(chain))
    }

    /// Update an address balance
    pub fn update_balance(&mut self, address: &Address, amount: u64) {
        let balance = Balance::new(amount, address.chain());
        self.set_balance(address, balance);
    }

    /// Add to an address balance
    pub fn add_to_balance(&mut self, address: &Address, amount: u64) {
        let current = self.get_balance(address);
        let new_balance = current.add(amount);
        self.set_balance(address, new_balance);
    }

    /// Get all balances
    pub fn all_balances(&self) -> &HashMap<String, Balance> {
        &self.balances
    }

    /// Update cached totals
    fn update_totals(&mut self) {
        self.totals.clear();
        for balance in self.balances.values() {
            let total = self.totals.entry(balance.chain()).or_insert(Balance::zero(balance.chain()));
            *total = total.add(balance.amount());
        }
    }

    /// Check if any balance exists
    pub fn has_balance(&self) -> bool {
        self.balances.values().any(|b| b.amount() > 0)
    }

    /// Get summary of all chain totals
    pub fn summary(&self) -> Vec<(Chain, Balance)> {
        let mut chains: Vec<_> = self.totals.iter().map(|(c, b)| (*c, *b)).collect();
        chains.sort_by_key(|(c, _)| c.chain_id());
        chains
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::address::AddressType;

    #[test]
    fn test_balance_creation() {
        let balance = Balance::new(100_000_000, Chain::BitCell);
        assert_eq!(balance.amount(), 100_000_000);
        assert_eq!(balance.chain(), Chain::BitCell);
    }

    #[test]
    fn test_balance_format() {
        // 1 CELL = 100_000_000 units (8 decimals)
        let balance = Balance::new(100_000_000, Chain::BitCell);
        assert_eq!(balance.format(), "1 CELL");
        
        let balance = Balance::new(150_000_000, Chain::BitCell);
        assert_eq!(balance.format(), "1.5 CELL");
        
        let balance = Balance::new(12_345_678, Chain::BitCell);
        assert_eq!(balance.format(), "0.12345678 CELL");
    }

    #[test]
    fn test_balance_format_fixed() {
        let balance = Balance::new(100_000_000, Chain::BitCell);
        assert_eq!(balance.format_fixed(2), "1.00 CELL");
        
        let balance = Balance::new(150_500_000, Chain::BitCell);
        assert_eq!(balance.format_fixed(4), "1.5050 CELL");
    }

    #[test]
    fn test_bitcoin_balance_format() {
        // 1 BTC = 100_000_000 satoshis
        let balance = Balance::new(100_000_000, Chain::Bitcoin);
        assert_eq!(balance.format(), "1 BTC");
        
        let balance = Balance::new(50_000, Chain::Bitcoin);
        assert_eq!(balance.format(), "0.0005 BTC");
    }

    #[test]
    fn test_ethereum_balance_format() {
        // 1 ETH = 1_000_000_000_000_000_000 wei (18 decimals)
        let balance = Balance::new(1_000_000_000_000_000_000, Chain::Ethereum);
        assert_eq!(balance.format(), "1 ETH");
        
        let balance = Balance::new(500_000_000_000_000_000, Chain::Ethereum);
        assert_eq!(balance.format(), "0.5 ETH");
    }

    #[test]
    fn test_balance_operations() {
        let balance = Balance::new(1000, Chain::BitCell);
        
        let added = balance.add(500);
        assert_eq!(added.amount(), 1500);
        
        let subtracted = balance.subtract(300).unwrap();
        assert_eq!(subtracted.amount(), 700);
        
        let insufficient = balance.subtract(2000);
        assert!(insufficient.is_none());
    }

    #[test]
    fn test_balance_sufficient() {
        let balance = Balance::new(1000, Chain::BitCell);
        
        assert!(balance.is_sufficient(500));
        assert!(balance.is_sufficient(1000));
        assert!(!balance.is_sufficient(1001));
    }

    #[test]
    fn test_from_units() {
        let balance = Balance::from_units(1.5, Chain::BitCell);
        assert_eq!(balance.amount(), 150_000_000);
        
        let balance = Balance::from_units(0.001, Chain::Bitcoin);
        assert_eq!(balance.amount(), 100_000);
    }

    #[test]
    fn test_to_units() {
        let balance = Balance::new(150_000_000, Chain::BitCell);
        assert!((balance.to_units() - 1.5).abs() < 0.0001);
    }

    #[test]
    fn test_balance_tracker() {
        let mut tracker = BalanceTracker::new();
        
        let address = Address::new(
            vec![0u8; 20],
            AddressType::BitCell,
            Chain::BitCell,
            0,
        );
        
        tracker.set_balance(&address, Balance::new(1000, Chain::BitCell));
        
        let balance = tracker.get_balance(&address);
        assert_eq!(balance.amount(), 1000);
    }

    #[test]
    fn test_balance_tracker_totals() {
        let mut tracker = BalanceTracker::new();
        
        let addr1 = Address::new(vec![1u8; 20], AddressType::BitCell, Chain::BitCell, 0);
        let addr2 = Address::new(vec![2u8; 20], AddressType::BitCell, Chain::BitCell, 1);
        
        tracker.set_balance(&addr1, Balance::new(1000, Chain::BitCell));
        tracker.set_balance(&addr2, Balance::new(2000, Chain::BitCell));
        
        let total = tracker.get_total(Chain::BitCell);
        assert_eq!(total.amount(), 3000);
    }

    #[test]
    fn test_zero_balance() {
        let balance = Balance::zero(Chain::Bitcoin);
        assert_eq!(balance.amount(), 0);
        assert_eq!(balance.format(), "0 BTC");
    }

    #[test]
    fn test_saturating_add() {
        let balance = Balance::new(u64::MAX - 10, Chain::BitCell);
        let added = balance.add(20);
        assert_eq!(added.amount(), u64::MAX);
    }
}
