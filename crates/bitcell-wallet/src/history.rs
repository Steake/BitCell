//! Transaction History
//!
//! Provides transaction history tracking and display.

use crate::{Chain, transaction::TransactionStatus};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Direction of a transaction
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TransactionDirection {
    /// Incoming transaction
    Incoming,
    /// Outgoing transaction
    Outgoing,
    /// Self transfer
    SelfTransfer,
}

/// Transaction record for history
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionRecord {
    /// Transaction hash
    pub tx_hash: String,
    /// Chain
    pub chain: Chain,
    /// Direction
    pub direction: TransactionDirection,
    /// Sender address
    pub from: String,
    /// Recipient address
    pub to: String,
    /// Amount in smallest units
    pub amount: u64,
    /// Fee in smallest units
    pub fee: u64,
    /// Status
    pub status: TransactionStatus,
    /// Block height (if confirmed)
    pub block_height: Option<u64>,
    /// Timestamp (Unix epoch)
    pub timestamp: u64,
    /// Confirmations count
    pub confirmations: u32,
    /// Optional memo/note
    pub memo: Option<String>,
}

impl TransactionRecord {
    /// Create a new transaction record
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        tx_hash: String,
        chain: Chain,
        direction: TransactionDirection,
        from: String,
        to: String,
        amount: u64,
        fee: u64,
        timestamp: u64,
    ) -> Self {
        Self {
            tx_hash,
            chain,
            direction,
            from,
            to,
            amount,
            fee,
            status: TransactionStatus::Pending,
            block_height: None,
            timestamp,
            confirmations: 0,
            memo: None,
        }
    }

    /// Set the transaction as confirmed
    pub fn confirm(&mut self, block_height: u64) {
        self.status = TransactionStatus::Confirmed;
        self.block_height = Some(block_height);
        self.confirmations = 1;
    }

    /// Update confirmations count
    pub fn update_confirmations(&mut self, current_height: u64) {
        if let Some(tx_height) = self.block_height {
            self.confirmations = (current_height.saturating_sub(tx_height) + 1) as u32;
        }
    }

    /// Check if transaction is fully confirmed (6+ confirmations for Bitcoin-like)
    pub fn is_fully_confirmed(&self) -> bool {
        match self.chain {
            Chain::BitCell => self.confirmations >= 6,
            Chain::Bitcoin | Chain::BitcoinTestnet => self.confirmations >= 6,
            Chain::Ethereum | Chain::EthereumSepolia => self.confirmations >= 12,
            Chain::Custom(_) => self.confirmations >= 1,
        }
    }

    /// Set memo
    pub fn with_memo(mut self, memo: &str) -> Self {
        self.memo = Some(memo.to_string());
        self
    }

    /// Get a short version of the tx hash
    /// 
    /// Returns a truncated hash in the format "prefix...suffix".
    /// Assumes the hash is an ASCII hex string.
    pub fn short_hash(&self) -> String {
        let chars: Vec<char> = self.tx_hash.chars().collect();
        if chars.len() > 16 {
            let prefix: String = chars[..8].iter().collect();
            let suffix: String = chars[chars.len()-8..].iter().collect();
            format!("{}...{}", prefix, suffix)
        } else {
            self.tx_hash.clone()
        }
    }

    /// Format amount for display
    pub fn format_amount(&self) -> String {
        let decimals = self.chain.decimals() as u32;
        let divisor = 10u64.pow(decimals);
        let whole = self.amount / divisor;
        let fraction = self.amount % divisor;
        
        let prefix = match self.direction {
            TransactionDirection::Incoming => "+",
            TransactionDirection::Outgoing => "-",
            TransactionDirection::SelfTransfer => "±",
        };
        
        if fraction == 0 {
            format!("{}{} {}", prefix, whole, self.chain.symbol())
        } else {
            let fraction_str = format!("{:0>width$}", fraction, width = decimals as usize);
            let trimmed = fraction_str.trim_end_matches('0');
            format!("{}{}.{} {}", prefix, whole, trimmed, self.chain.symbol())
        }
    }

    /// Format date for display
    /// 
    /// Returns a human-readable date string in the format "YYYY-MM-DD HH:MM:SS UTC".
    pub fn format_date(&self) -> String {
        // Convert Unix timestamp to date components
        const SECONDS_PER_MINUTE: u64 = 60;
        const SECONDS_PER_HOUR: u64 = 3600;
        const SECONDS_PER_DAY: u64 = 86400;
        
        let mut remaining = self.timestamp;
        
        // Calculate days since epoch
        let days = remaining / SECONDS_PER_DAY;
        remaining %= SECONDS_PER_DAY;
        
        // Calculate time components
        let hours = remaining / SECONDS_PER_HOUR;
        remaining %= SECONDS_PER_HOUR;
        let minutes = remaining / SECONDS_PER_MINUTE;
        let seconds = remaining % SECONDS_PER_MINUTE;
        
        // Calculate year, month, day from days since epoch (1970-01-01)
        let (year, month, day) = days_to_ymd(days);
        
        format!("{:04}-{:02}-{:02} {:02}:{:02}:{:02} UTC", year, month, day, hours, minutes, seconds)
    }
}

/// Convert days since epoch to year, month, day
fn days_to_ymd(days: u64) -> (u64, u64, u64) {
    // Simplified calculation - handles dates from 1970 onwards
    let mut remaining_days = days;
    let mut year = 1970u64;
    
    loop {
        let days_in_year = if is_leap_year(year) { 366 } else { 365 };
        if remaining_days < days_in_year {
            break;
        }
        remaining_days -= days_in_year;
        year += 1;
    }
    
    let days_in_months: [u64; 12] = if is_leap_year(year) {
        [31, 29, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
    } else {
        [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
    };
    
    let mut month = 1u64;
    for days_in_month in days_in_months.iter() {
        if remaining_days < *days_in_month {
            break;
        }
        remaining_days -= *days_in_month;
        month += 1;
    }
    
    let day = remaining_days + 1;
    (year, month, day)
}

/// Check if a year is a leap year
fn is_leap_year(year: u64) -> bool {
    (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0)
}

/// Transaction history manager
/// 
/// Note: When deserializing, call `rebuild_index()` to ensure
/// the hash index is correctly populated for lookups.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TransactionHistory {
    /// All transactions
    transactions: Vec<TransactionRecord>,
    /// Index by tx hash for quick lookup
    #[serde(skip)]
    hash_index: HashMap<String, usize>,
}

impl TransactionHistory {
    /// Create a new empty history
    pub fn new() -> Self {
        Self {
            transactions: Vec::new(),
            hash_index: HashMap::new(),
        }
    }

    /// Rebuild the hash index after deserialization
    /// 
    /// Call this method after deserializing a TransactionHistory to ensure
    /// lookups via get() and get_mut() work correctly.
    pub fn ensure_indexed(&mut self) {
        if self.hash_index.is_empty() && !self.transactions.is_empty() {
            self.rebuild_index();
        }
    }

    /// Add a transaction to history
    pub fn add(&mut self, record: TransactionRecord) {
        let hash = record.tx_hash.clone();
        let idx = self.transactions.len();
        self.transactions.push(record);
        self.hash_index.insert(hash, idx);
    }

    /// Get transaction by hash
    pub fn get(&self, tx_hash: &str) -> Option<&TransactionRecord> {
        self.hash_index.get(tx_hash).map(|&idx| &self.transactions[idx])
    }

    /// Get mutable transaction by hash
    pub fn get_mut(&mut self, tx_hash: &str) -> Option<&mut TransactionRecord> {
        self.hash_index.get(tx_hash).copied().map(|idx| &mut self.transactions[idx])
    }

    /// Get all transactions
    pub fn all(&self) -> &[TransactionRecord] {
        &self.transactions
    }

    /// Get transactions for a specific chain
    pub fn by_chain(&self, chain: Chain) -> Vec<&TransactionRecord> {
        self.transactions.iter().filter(|tx| tx.chain == chain).collect()
    }

    /// Get transactions for a specific address
    pub fn by_address(&self, address: &str) -> Vec<&TransactionRecord> {
        self.transactions.iter()
            .filter(|tx| tx.from == address || tx.to == address)
            .collect()
    }

    /// Get pending transactions
    pub fn pending(&self) -> Vec<&TransactionRecord> {
        self.transactions.iter()
            .filter(|tx| tx.status == TransactionStatus::Pending)
            .collect()
    }

    /// Get recent transactions (last n)
    pub fn recent(&self, n: usize) -> Vec<&TransactionRecord> {
        let start = self.transactions.len().saturating_sub(n);
        self.transactions[start..].iter().collect()
    }

    /// Get transactions sorted by timestamp (newest first)
    pub fn sorted_by_time(&self) -> Vec<&TransactionRecord> {
        let mut txs: Vec<_> = self.transactions.iter().collect();
        txs.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
        txs
    }

    /// Count transactions
    pub fn count(&self) -> usize {
        self.transactions.len()
    }

    /// Count by status
    pub fn count_by_status(&self, status: TransactionStatus) -> usize {
        self.transactions.iter().filter(|tx| tx.status == status).count()
    }

    /// Update confirmations for all transactions
    pub fn update_confirmations(&mut self, chain: Chain, current_height: u64) {
        for tx in &mut self.transactions {
            if tx.chain == chain {
                tx.update_confirmations(current_height);
            }
        }
    }

    /// Remove old completed transactions (keep last n)
    pub fn prune(&mut self, keep_count: usize) {
        if self.transactions.len() > keep_count {
            let to_remove = self.transactions.len() - keep_count;
            // Remove confirmed transactions first
            let mut removed = 0;
            self.transactions.retain(|tx| {
                if removed >= to_remove {
                    return true;
                }
                if tx.status == TransactionStatus::Confirmed && tx.is_fully_confirmed() {
                    removed += 1;
                    false
                } else {
                    true
                }
            });
            self.rebuild_index();
        }
    }

    /// Rebuild the hash index
    fn rebuild_index(&mut self) {
        self.hash_index.clear();
        for (idx, tx) in self.transactions.iter().enumerate() {
            self.hash_index.insert(tx.tx_hash.clone(), idx);
        }
    }

    /// Get summary statistics
    pub fn summary(&self) -> HistorySummary {
        let total = self.transactions.len();
        let pending = self.count_by_status(TransactionStatus::Pending);
        let confirmed = self.count_by_status(TransactionStatus::Confirmed);
        let failed = self.count_by_status(TransactionStatus::Failed);
        
        let (total_sent, total_received) = self.transactions.iter().fold((0u64, 0u64), |(sent, recv), tx| {
            match tx.direction {
                TransactionDirection::Outgoing => (sent.saturating_add(tx.amount), recv),
                TransactionDirection::Incoming => (sent, recv.saturating_add(tx.amount)),
                TransactionDirection::SelfTransfer => (sent, recv),
            }
        });
        
        HistorySummary {
            total_transactions: total,
            pending_transactions: pending,
            confirmed_transactions: confirmed,
            failed_transactions: failed,
            total_sent,
            total_received,
        }
    }
}

/// Summary of transaction history
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistorySummary {
    pub total_transactions: usize,
    pub pending_transactions: usize,
    pub confirmed_transactions: usize,
    pub failed_transactions: usize,
    pub total_sent: u64,
    pub total_received: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_record(hash: &str, direction: TransactionDirection, amount: u64) -> TransactionRecord {
        TransactionRecord::new(
            hash.to_string(),
            Chain::BitCell,
            direction,
            "BC1sender".to_string(),
            "BC1receiver".to_string(),
            amount,
            100,
            1234567890,
        )
    }

    #[test]
    fn test_transaction_record_creation() {
        let record = create_test_record("0x123", TransactionDirection::Outgoing, 1000);
        
        assert_eq!(record.tx_hash, "0x123");
        assert_eq!(record.amount, 1000);
        assert_eq!(record.status, TransactionStatus::Pending);
        assert_eq!(record.confirmations, 0);
    }

    #[test]
    fn test_transaction_confirm() {
        let mut record = create_test_record("0x123", TransactionDirection::Outgoing, 1000);
        
        record.confirm(100);
        
        assert_eq!(record.status, TransactionStatus::Confirmed);
        assert_eq!(record.block_height, Some(100));
        assert_eq!(record.confirmations, 1);
    }

    #[test]
    fn test_update_confirmations() {
        let mut record = create_test_record("0x123", TransactionDirection::Outgoing, 1000);
        record.confirm(100);
        
        record.update_confirmations(105);
        
        assert_eq!(record.confirmations, 6);
    }

    #[test]
    fn test_is_fully_confirmed() {
        let mut record = create_test_record("0x123", TransactionDirection::Outgoing, 1000);
        record.confirm(100);
        
        record.confirmations = 5;
        assert!(!record.is_fully_confirmed());
        
        record.confirmations = 6;
        assert!(record.is_fully_confirmed());
    }

    #[test]
    fn test_format_amount() {
        let record = TransactionRecord::new(
            "0x123".to_string(),
            Chain::BitCell,
            TransactionDirection::Incoming,
            "BC1sender".to_string(),
            "BC1receiver".to_string(),
            100_000_000, // 1 CELL
            100,
            1234567890,
        );
        
        assert!(record.format_amount().starts_with('+'));
    }

    #[test]
    fn test_short_hash() {
        let record = create_test_record(
            "0x1234567890abcdef1234567890abcdef",
            TransactionDirection::Outgoing,
            1000,
        );
        
        let short = record.short_hash();
        assert!(short.contains("..."));
        assert!(short.len() < record.tx_hash.len());
    }

    #[test]
    fn test_history_add_and_get() {
        let mut history = TransactionHistory::new();
        let record = create_test_record("0x123", TransactionDirection::Outgoing, 1000);
        
        history.add(record);
        
        let retrieved = history.get("0x123").unwrap();
        assert_eq!(retrieved.amount, 1000);
    }

    #[test]
    fn test_history_by_chain() {
        let mut history = TransactionHistory::new();
        
        history.add(create_test_record("0x1", TransactionDirection::Outgoing, 100));
        
        let mut btc_record = create_test_record("0x2", TransactionDirection::Incoming, 200);
        btc_record.chain = Chain::Bitcoin;
        history.add(btc_record);
        
        let bitcell_txs = history.by_chain(Chain::BitCell);
        assert_eq!(bitcell_txs.len(), 1);
        
        let btc_txs = history.by_chain(Chain::Bitcoin);
        assert_eq!(btc_txs.len(), 1);
    }

    #[test]
    fn test_history_by_address() {
        let mut history = TransactionHistory::new();
        
        history.add(create_test_record("0x1", TransactionDirection::Outgoing, 100));
        history.add(create_test_record("0x2", TransactionDirection::Incoming, 200));
        
        let txs = history.by_address("BC1sender");
        assert_eq!(txs.len(), 2);
    }

    #[test]
    fn test_history_pending() {
        let mut history = TransactionHistory::new();
        
        history.add(create_test_record("0x1", TransactionDirection::Outgoing, 100));
        
        let mut confirmed = create_test_record("0x2", TransactionDirection::Incoming, 200);
        confirmed.status = TransactionStatus::Confirmed;
        history.add(confirmed);
        
        let pending = history.pending();
        assert_eq!(pending.len(), 1);
    }

    #[test]
    fn test_history_recent() {
        let mut history = TransactionHistory::new();
        
        for i in 0..10 {
            history.add(create_test_record(
                &format!("0x{}", i),
                TransactionDirection::Outgoing,
                100 * i,
            ));
        }
        
        let recent = history.recent(5);
        assert_eq!(recent.len(), 5);
    }

    #[test]
    fn test_history_summary() {
        let mut history = TransactionHistory::new();
        
        history.add(create_test_record("0x1", TransactionDirection::Outgoing, 100));
        history.add(create_test_record("0x2", TransactionDirection::Incoming, 200));
        
        let summary = history.summary();
        
        assert_eq!(summary.total_transactions, 2);
        assert_eq!(summary.total_sent, 100);
        assert_eq!(summary.total_received, 200);
    }

    #[test]
    fn test_transaction_with_memo() {
        let record = create_test_record("0x123", TransactionDirection::Outgoing, 1000)
            .with_memo("Payment for services");
        
        assert_eq!(record.memo, Some("Payment for services".to_string()));
    }
}
