//! Retry policies for fetch operations.

use std::time::Duration;

/// Backoff strategy between retry attempts.
#[derive(Debug, Clone)]
pub enum BackoffStrategy {
    /// No delay between retries.
    None,
    /// Fixed delay between retries.
    Fixed(Duration),
    /// Exponential backoff with base and max.
    Exponential {
        /// Initial delay.
        base: Duration,
        /// Maximum delay.
        max: Duration,
    },
}

impl BackoffStrategy {
    /// Calculate delay for a given attempt number (0-indexed).
    pub fn delay_for_attempt(&self, attempt: u32) -> Duration {
        match self {
            Self::None => Duration::ZERO,
            Self::Fixed(d) => *d,
            Self::Exponential { base, max } => {
                let multiplier = 2u64.saturating_pow(attempt);
                let delay = Duration::from_millis(
                    base.as_millis() as u64 * multiplier
                );
                std::cmp::min(delay, *max)
            }
        }
    }
}

impl Default for BackoffStrategy {
    fn default() -> Self {
        Self::Exponential {
            base: Duration::from_millis(50),
            max: Duration::from_millis(500),
        }
    }
}

/// Conditions that trigger a retry.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RetryCondition {
    /// Retry on specific HTTP status code.
    StatusCode(u16),
    /// Retry on any 5xx status.
    ServerError,
    /// Retry on timeout.
    Timeout,
    /// Retry on connection error.
    ConnectionError,
}

impl RetryCondition {
    /// Check if a status code matches this condition.
    pub fn matches_status(&self, status: u16) -> bool {
        match self {
            Self::StatusCode(code) => status == *code,
            Self::ServerError => (500..600).contains(&status),
            _ => false,
        }
    }
}

/// Retry policy configuration.
#[derive(Debug, Clone)]
pub struct RetryPolicy {
    /// Maximum number of retry attempts.
    pub max_attempts: u32,
    /// Backoff strategy.
    pub backoff: BackoffStrategy,
    /// Conditions that trigger retry.
    pub retry_on: Vec<RetryCondition>,
}

impl RetryPolicy {
    /// Create a new retry policy.
    pub fn new(max_attempts: u32) -> Self {
        Self {
            max_attempts,
            backoff: BackoffStrategy::default(),
            retry_on: vec![
                RetryCondition::ServerError,
                RetryCondition::Timeout,
                RetryCondition::ConnectionError,
            ],
        }
    }

    /// Create a policy with no retries.
    pub fn none() -> Self {
        Self {
            max_attempts: 0,
            backoff: BackoffStrategy::None,
            retry_on: Vec::new(),
        }
    }

    /// Set backoff strategy.
    pub fn with_backoff(mut self, strategy: BackoffStrategy) -> Self {
        self.backoff = strategy;
        self
    }

    /// Set retry conditions.
    pub fn with_conditions(mut self, conditions: Vec<RetryCondition>) -> Self {
        self.retry_on = conditions;
        self
    }

    /// Check if should retry based on status code.
    pub fn should_retry_status(&self, status: u16, attempt: u32) -> bool {
        if attempt >= self.max_attempts {
            return false;
        }
        self.retry_on.iter().any(|c| c.matches_status(status))
    }

    /// Check if should retry on timeout.
    pub fn should_retry_timeout(&self, attempt: u32) -> bool {
        if attempt >= self.max_attempts {
            return false;
        }
        self.retry_on.contains(&RetryCondition::Timeout)
    }
}

impl Default for RetryPolicy {
    fn default() -> Self {
        Self::new(1)
    }
}
