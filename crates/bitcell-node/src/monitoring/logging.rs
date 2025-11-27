//! Structured logging for BitCell nodes

use std::fmt;

/// Log levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum LogLevel {
    Debug,
    Info,
    Warn,
    Error,
}

impl fmt::Display for LogLevel {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            LogLevel::Debug => write!(f, "DEBUG"),
            LogLevel::Info => write!(f, "INFO"),
            LogLevel::Warn => write!(f, "WARN"),
            LogLevel::Error => write!(f, "ERROR"),
        }
    }
}

/// Structured log event
#[derive(Debug, Clone)]
pub struct LogEvent {
    pub level: LogLevel,
    pub module: String,
    pub message: String,
    pub timestamp: u64,
}

impl LogEvent {
    pub fn new(level: LogLevel, module: &str, message: &str) -> Self {
        Self {
            level,
            module: module.to_string(),
            message: message.to_string(),
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        }
    }
    
    /// Format as JSON for structured logging
    pub fn to_json(&self) -> String {
        format!(
            r#"{{"level":"{}","module":"{}","message":"{}","timestamp":{}}}"#,
            self.level,
            self.module,
            self.message.replace('"', "\\\""),
            self.timestamp
        )
    }
    
    /// Format for human-readable console output
    pub fn to_console(&self) -> String {
        format!(
            "[{}] [{}] {}",
            self.level,
            self.module,
            self.message
        )
    }
}

/// Simple logger that can output to console or JSON
pub struct Logger {
    min_level: LogLevel,
    json_format: bool,
}

impl Logger {
    pub fn new(min_level: LogLevel, json_format: bool) -> Self {
        Self { min_level, json_format }
    }
    
    pub fn log(&self, event: LogEvent) {
        if event.level >= self.min_level {
            let output = if self.json_format {
                event.to_json()
            } else {
                event.to_console()
            };
            println!("{}", output);
        }
    }
    
    pub fn debug(&self, module: &str, message: &str) {
        self.log(LogEvent::new(LogLevel::Debug, module, message));
    }
    
    pub fn info(&self, module: &str, message: &str) {
        self.log(LogEvent::new(LogLevel::Info, module, message));
    }
    
    pub fn warn(&self, module: &str, message: &str) {
        self.log(LogEvent::new(LogLevel::Warn, module, message));
    }
    
    pub fn error(&self, module: &str, message: &str) {
        self.log(LogEvent::new(LogLevel::Error, module, message));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_log_event() {
        let event = LogEvent::new(LogLevel::Info, "test", "Hello");
        assert_eq!(event.level, LogLevel::Info);
        assert_eq!(event.module, "test");
        assert_eq!(event.message, "Hello");
    }

    #[test]
    fn test_log_event_json() {
        let event = LogEvent::new(LogLevel::Error, "network", "Connection failed");
        let json = event.to_json();
        assert!(json.contains(r#""level":"ERROR""#));
        assert!(json.contains(r#""module":"network""#));
        assert!(json.contains(r#""message":"Connection failed""#));
    }

    #[test]
    fn test_log_event_console() {
        let event = LogEvent::new(LogLevel::Warn, "consensus", "Fork detected");
        let console = event.to_console();
        assert!(console.contains("[WARN]"));
        assert!(console.contains("[consensus]"));
        assert!(console.contains("Fork detected"));
    }

    #[test]
    fn test_logger_filtering() {
        let logger = Logger::new(LogLevel::Warn, false);
        
        // These should be printed (level >= Warn)
        logger.warn("test", "This is a warning");
        logger.error("test", "This is an error");
        
        // These should NOT be printed (level < Warn)
        logger.debug("test", "This is debug");
        logger.info("test", "This is info");
    }
}
