//! Request lifecycle tracking.

use std::collections::HashMap;
use std::time::{Duration, Instant};

/// Lifecycle phases for a request.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LifecyclePhase {
    /// Request received, processing started.
    Start,
    /// Shell HTML has been flushed to client.
    ShellSent,
    /// A named section has been sent.
    SectionSent(String),
    /// Request completed successfully.
    Completion,
    /// An error occurred.
    Error(String),
}

/// Timing context for observability.
#[derive(Debug, Clone)]
pub struct TimingContext {
    start: Instant,
    marks: HashMap<String, Instant>,
}

impl TimingContext {
    /// Create a new timing context.
    pub fn new() -> Self {
        Self {
            start: Instant::now(),
            marks: HashMap::new(),
        }
    }

    /// Record a timing mark.
    pub fn mark(&mut self, name: &str) {
        self.marks.insert(name.to_string(), Instant::now());
    }

    /// Mark section start.
    pub fn mark_section_start(&mut self, section: &str) {
        self.mark(&format!("section_{}_start", section));
    }

    /// Mark section sent.
    pub fn mark_section_sent(&mut self, section: &str) {
        self.mark(&format!("section_{}_sent", section));
    }

    /// Get elapsed time since start.
    pub fn elapsed(&self) -> Duration {
        self.start.elapsed()
    }

    /// Get time to shell flush.
    pub fn time_to_shell(&self) -> Option<Duration> {
        self.marks
            .get("shell_sent")
            .map(|t| t.duration_since(self.start))
    }

    /// Get time to first section.
    pub fn time_to_first_section(&self) -> Option<Duration> {
        self.marks
            .iter()
            .filter(|(k, _)| k.ends_with("_sent") && k.starts_with("section_"))
            .map(|(_, t)| t.duration_since(self.start))
            .min()
    }

    /// Get total request time.
    pub fn total_time(&self) -> Duration {
        self.elapsed()
    }

    /// Get timing for a specific section.
    pub fn section_timing(&self, section: &str) -> Option<SectionTiming> {
        let start_key = format!("section_{}_start", section);
        let sent_key = format!("section_{}_sent", section);

        let start = self.marks.get(&start_key)?;
        let sent = self.marks.get(&sent_key)?;

        Some(SectionTiming {
            name: section.to_string(),
            start: start.duration_since(self.start),
            sent: sent.duration_since(self.start),
            duration: sent.duration_since(*start),
        })
    }
}

impl Default for TimingContext {
    fn default() -> Self {
        Self::new()
    }
}

/// Timing information for a section.
#[derive(Debug, Clone)]
pub struct SectionTiming {
    /// Section name.
    pub name: String,
    /// Time from request start to section start.
    pub start: Duration,
    /// Time from request start to section sent.
    pub sent: Duration,
    /// Duration of section rendering.
    pub duration: Duration,
}

/// Observer trait for lifecycle events.
pub trait LifecycleObserver: Send + Sync {
    /// Called when a lifecycle phase occurs.
    fn on_phase(&self, phase: LifecyclePhase, elapsed: Duration);
}
