//! Explicit flush control - no implicit buffering.

/// Flush policy for streaming responses.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum FlushPolicy {
    /// Flush immediately after shell is sent.
    #[default]
    AfterShell,
    /// Flush after each section is sent.
    AfterEachSection,
    /// Manual flush control only.
    Manual,
}

impl FlushPolicy {
    /// Check if should flush after shell.
    pub fn flush_after_shell(&self) -> bool {
        matches!(self, Self::AfterShell | Self::AfterEachSection)
    }

    /// Check if should flush after section.
    pub fn flush_after_section(&self) -> bool {
        matches!(self, Self::AfterEachSection)
    }
}

/// Controller for managing flush behavior.
#[derive(Debug)]
pub struct FlushController {
    policy: FlushPolicy,
    pending_bytes: usize,
    /// Maximum bytes to buffer (0 = immediate flush).
    max_buffer: usize,
}

impl FlushController {
    /// Create a new flush controller with given policy.
    pub fn new(policy: FlushPolicy) -> Self {
        Self {
            policy,
            pending_bytes: 0,
            max_buffer: 0, // Default: immediate flush
        }
    }

    /// Set maximum buffer size before auto-flush.
    pub fn with_max_buffer(mut self, bytes: usize) -> Self {
        self.max_buffer = bytes;
        self
    }

    /// Record bytes added to buffer.
    pub fn add_bytes(&mut self, count: usize) {
        self.pending_bytes += count;
    }

    /// Check if flush is needed.
    pub fn should_flush(&self) -> bool {
        self.max_buffer == 0 || self.pending_bytes >= self.max_buffer
    }

    /// Reset pending byte count after flush.
    pub fn reset(&mut self) {
        self.pending_bytes = 0;
    }

    /// Get current policy.
    pub fn policy(&self) -> FlushPolicy {
        self.policy
    }
}

impl Default for FlushController {
    fn default() -> Self {
        Self::new(FlushPolicy::default())
    }
}
