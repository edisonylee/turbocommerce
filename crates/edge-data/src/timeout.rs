//! Timeout configuration for fetch operations.

use std::time::Duration;

/// Timeout configuration for a fetch operation.
#[derive(Debug, Clone)]
pub struct TimeoutConfig {
    /// Connection timeout.
    pub connect: Duration,
    /// Time to first byte.
    pub response: Duration,
    /// Total operation timeout.
    pub total: Duration,
}

impl TimeoutConfig {
    /// Create a new timeout configuration.
    pub fn new(connect: Duration, response: Duration, total: Duration) -> Self {
        Self {
            connect,
            response,
            total,
        }
    }

    /// Create from a single total timeout.
    pub fn from_total(total: Duration) -> Self {
        Self {
            connect: Duration::from_millis(total.as_millis() as u64 / 4),
            response: Duration::from_millis(total.as_millis() as u64 / 2),
            total,
        }
    }

    /// Create with aggressive timeouts (for latency-critical paths).
    pub fn aggressive(total: Duration) -> Self {
        Self {
            connect: Duration::from_millis(50),
            response: Duration::from_millis(100),
            total,
        }
    }
}

impl Default for TimeoutConfig {
    fn default() -> Self {
        Self {
            connect: Duration::from_millis(100),
            response: Duration::from_millis(200),
            total: Duration::from_millis(500),
        }
    }
}

/// Error when a timeout is exceeded.
#[derive(Debug, Clone, thiserror::Error)]
pub enum TimeoutError {
    #[error("Connection timeout after {0:?}")]
    Connect(Duration),

    #[error("Response timeout after {0:?}")]
    Response(Duration),

    #[error("Total timeout after {0:?}")]
    Total(Duration),
}
