//! Audit logging for admin console actions
//!
//! Tracks all administrative actions for security and compliance.

use chrono::{DateTime, Utc};
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

/// Maximum number of audit log entries to keep in memory
const MAX_AUDIT_LOGS: usize = 10_000;

/// Audit log entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditLogEntry {
    pub id: String,
    pub timestamp: DateTime<Utc>,
    pub user_id: String,
    pub username: String,
    pub action: String,
    pub resource: String,
    pub details: Option<String>,
    pub ip_address: Option<String>,
    pub success: bool,
    pub error_message: Option<String>,
}

/// Audit logger
pub struct AuditLogger {
    logs: RwLock<VecDeque<AuditLogEntry>>,
}

impl AuditLogger {
    /// Create a new audit logger
    pub fn new() -> Self {
        Self {
            logs: RwLock::new(VecDeque::with_capacity(MAX_AUDIT_LOGS)),
        }
    }

    /// Log an action
    pub fn log(
        &self,
        user_id: String,
        username: String,
        action: String,
        resource: String,
        details: Option<String>,
        success: bool,
        error_message: Option<String>,
    ) {
        let entry = AuditLogEntry {
            id: uuid::Uuid::new_v4().to_string(),
            timestamp: Utc::now(),
            user_id,
            username: username.clone(),
            action: action.clone(),
            resource: resource.clone(),
            details,
            ip_address: None, // TODO: Extract from request
            success,
            error_message: error_message.clone(),
        };

        let mut logs = self.logs.write();
        
        // Remove oldest entry if at capacity
        if logs.len() >= MAX_AUDIT_LOGS {
            logs.pop_front();
        }
        
        logs.push_back(entry.clone());

        // Also log to tracing for immediate visibility
        if success {
            tracing::info!(
                user = %username,
                action = %action,
                resource = %resource,
                "Audit: {} performed {} on {}",
                username, action, resource
            );
        } else {
            tracing::warn!(
                user = %username,
                action = %action,
                resource = %resource,
                error = ?error_message,
                "Audit: {} failed to perform {} on {}",
                username, action, resource
            );
        }
    }

    /// Log a successful action
    pub fn log_success(
        &self,
        user_id: String,
        username: String,
        action: String,
        resource: String,
        details: Option<String>,
    ) {
        self.log(user_id, username, action, resource, details, true, None);
    }

    /// Log a failed action
    pub fn log_failure(
        &self,
        user_id: String,
        username: String,
        action: String,
        resource: String,
        error: String,
    ) {
        self.log(user_id, username, action, resource, None, false, Some(error));
    }

    /// Get all audit logs
    pub fn get_logs(&self) -> Vec<AuditLogEntry> {
        self.logs.read().iter().cloned().collect()
    }

    /// Get logs filtered by user
    pub fn get_logs_by_user(&self, user_id: &str) -> Vec<AuditLogEntry> {
        self.logs
            .read()
            .iter()
            .filter(|log| log.user_id == user_id)
            .cloned()
            .collect()
    }

    /// Get logs filtered by action
    pub fn get_logs_by_action(&self, action: &str) -> Vec<AuditLogEntry> {
        self.logs
            .read()
            .iter()
            .filter(|log| log.action == action)
            .cloned()
            .collect()
    }

    /// Get logs within a time range
    pub fn get_logs_by_time_range(
        &self,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> Vec<AuditLogEntry> {
        self.logs
            .read()
            .iter()
            .filter(|log| log.timestamp >= start && log.timestamp <= end)
            .cloned()
            .collect()
    }

    /// Get recent logs (last N entries)
    pub fn get_recent_logs(&self, count: usize) -> Vec<AuditLogEntry> {
        let logs = self.logs.read();
        let start = logs.len().saturating_sub(count);
        logs.iter().skip(start).cloned().collect()
    }

    /// Clear all logs (admin only)
    pub fn clear_logs(&self) {
        self.logs.write().clear();
        tracing::warn!("Audit logs cleared");
    }

    /// Get total log count
    pub fn count(&self) -> usize {
        self.logs.read().len()
    }
}

impl Default for AuditLogger {
    fn default() -> Self {
        Self::new()
    }
}

/// Helper macro for logging audit events
#[macro_export]
macro_rules! audit_log {
    ($logger:expr, $user_id:expr, $username:expr, $action:expr, $resource:expr) => {
        $logger.log_success(
            $user_id.to_string(),
            $username.to_string(),
            $action.to_string(),
            $resource.to_string(),
            None,
        )
    };
    ($logger:expr, $user_id:expr, $username:expr, $action:expr, $resource:expr, $details:expr) => {
        $logger.log_success(
            $user_id.to_string(),
            $username.to_string(),
            $action.to_string(),
            $resource.to_string(),
            Some($details.to_string()),
        )
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_audit_logger_creation() {
        let logger = AuditLogger::new();
        assert_eq!(logger.count(), 0);
    }

    #[test]
    fn test_log_success() {
        let logger = AuditLogger::new();
        logger.log_success(
            "user1".to_string(),
            "admin".to_string(),
            "start_node".to_string(),
            "node1".to_string(),
            Some("Node started successfully".to_string()),
        );

        let logs = logger.get_logs();
        assert_eq!(logs.len(), 1);
        assert_eq!(logs[0].user_id, "user1");
        assert_eq!(logs[0].username, "admin");
        assert_eq!(logs[0].action, "start_node");
        assert_eq!(logs[0].resource, "node1");
        assert!(logs[0].success);
    }

    #[test]
    fn test_log_failure() {
        let logger = AuditLogger::new();
        logger.log_failure(
            "user1".to_string(),
            "admin".to_string(),
            "delete_node".to_string(),
            "node1".to_string(),
            "Node not found".to_string(),
        );

        let logs = logger.get_logs();
        assert_eq!(logs.len(), 1);
        assert!(!logs[0].success);
        assert_eq!(logs[0].error_message, Some("Node not found".to_string()));
    }

    #[test]
    fn test_get_logs_by_user() {
        let logger = AuditLogger::new();
        logger.log_success(
            "user1".to_string(),
            "admin".to_string(),
            "start_node".to_string(),
            "node1".to_string(),
            None,
        );
        logger.log_success(
            "user2".to_string(),
            "operator".to_string(),
            "stop_node".to_string(),
            "node2".to_string(),
            None,
        );

        let user1_logs = logger.get_logs_by_user("user1");
        assert_eq!(user1_logs.len(), 1);
        assert_eq!(user1_logs[0].user_id, "user1");
    }

    #[test]
    fn test_get_logs_by_action() {
        let logger = AuditLogger::new();
        logger.log_success(
            "user1".to_string(),
            "admin".to_string(),
            "start_node".to_string(),
            "node1".to_string(),
            None,
        );
        logger.log_success(
            "user1".to_string(),
            "admin".to_string(),
            "start_node".to_string(),
            "node2".to_string(),
            None,
        );
        logger.log_success(
            "user1".to_string(),
            "admin".to_string(),
            "stop_node".to_string(),
            "node3".to_string(),
            None,
        );

        let start_logs = logger.get_logs_by_action("start_node");
        assert_eq!(start_logs.len(), 2);
    }

    #[test]
    fn test_recent_logs() {
        let logger = AuditLogger::new();
        for i in 0..10 {
            logger.log_success(
                "user1".to_string(),
                "admin".to_string(),
                format!("action{}", i),
                format!("resource{}", i),
                None,
            );
        }

        let recent = logger.get_recent_logs(5);
        assert_eq!(recent.len(), 5);
        assert_eq!(recent[0].action, "action5");
    }

    #[test]
    fn test_max_logs_rotation() {
        let logger = AuditLogger::new();
        
        // Add more than MAX_AUDIT_LOGS entries
        for i in 0..MAX_AUDIT_LOGS + 100 {
            logger.log_success(
                "user1".to_string(),
                "admin".to_string(),
                format!("action{}", i),
                "resource".to_string(),
                None,
            );
        }

        // Should only keep MAX_AUDIT_LOGS entries
        assert_eq!(logger.count(), MAX_AUDIT_LOGS);
        
        // Oldest entries should be removed
        let logs = logger.get_logs();
        assert_eq!(logs[0].action, "action100");
    }

    #[test]
    fn test_clear_logs() {
        let logger = AuditLogger::new();
        logger.log_success(
            "user1".to_string(),
            "admin".to_string(),
            "action".to_string(),
            "resource".to_string(),
            None,
        );

        assert_eq!(logger.count(), 1);
        logger.clear_logs();
        assert_eq!(logger.count(), 0);
    }
}
