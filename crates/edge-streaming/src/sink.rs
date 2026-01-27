//! Platform-controlled streaming sink.

use std::fmt::Display;

use edge_core::{LifecyclePhase, TimingContext, WorkloadError};
use futures::{Sink, SinkExt};

/// State of the streaming sink.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SinkState {
    /// Initial state, shell not yet sent.
    Initial,
    /// Shell has been sent, sections can be streamed.
    ShellSent,
    /// Response has been completed.
    Completed,
}

/// Platform-controlled streaming sink that enforces shell-first pattern.
///
/// This is generic over the underlying sink type to work with any
/// `Sink<Vec<u8>>` implementation, including Spin's `OutgoingBody`.
pub struct StreamingSink<S, E>
where
    S: Sink<Vec<u8>, Error = E> + Unpin,
    E: Display,
{
    inner: S,
    state: SinkState,
    timing: TimingContext,
    sections_sent: Vec<String>,
}

impl<S, E> StreamingSink<S, E>
where
    S: Sink<Vec<u8>, Error = E> + Unpin,
    E: Display,
{
    /// Create a new streaming sink.
    pub fn new(sink: S, timing: TimingContext) -> Self {
        Self {
            inner: sink,
            state: SinkState::Initial,
            timing,
            sections_sent: Vec::new(),
        }
    }

    /// Send the shell HTML. Must be called before any sections.
    ///
    /// The shell is the initial HTML structure that wraps sections.
    /// It should contain the doctype, head, and body structure.
    pub async fn send_shell(&mut self, html: &str) -> Result<(), WorkloadError> {
        if self.state != SinkState::Initial {
            return Err(WorkloadError::StreamError(
                "Shell already sent or sink completed".to_string(),
            ));
        }

        self.timing.mark("shell_start");
        self.inner
            .send(html.as_bytes().to_vec())
            .await
            .map_err(|e| WorkloadError::StreamError(e.to_string()))?;
        self.timing.mark("shell_sent");
        self.state = SinkState::ShellSent;

        Ok(())
    }

    /// Send a named section. Shell must be sent first.
    ///
    /// Sections are independently streamable parts of the page.
    /// They can be sent in any order after the shell.
    pub async fn send_section(&mut self, name: &str, html: &str) -> Result<(), WorkloadError> {
        if self.state == SinkState::Initial {
            return Err(WorkloadError::ShellNotSent);
        }
        if self.state == SinkState::Completed {
            return Err(WorkloadError::StreamError(
                "Sink already completed".to_string(),
            ));
        }

        self.timing.mark_section_start(name);
        self.inner
            .send(html.as_bytes().to_vec())
            .await
            .map_err(|e| WorkloadError::StreamError(e.to_string()))?;
        self.timing.mark_section_sent(name);
        self.sections_sent.push(name.to_string());

        Ok(())
    }

    /// Send raw bytes. Shell must be sent first.
    pub async fn send_raw(&mut self, bytes: Vec<u8>) -> Result<(), WorkloadError> {
        if self.state == SinkState::Initial {
            return Err(WorkloadError::ShellNotSent);
        }
        if self.state == SinkState::Completed {
            return Err(WorkloadError::StreamError(
                "Sink already completed".to_string(),
            ));
        }

        self.inner
            .send(bytes)
            .await
            .map_err(|e| WorkloadError::StreamError(e.to_string()))?;

        Ok(())
    }

    /// Complete the response.
    pub fn complete(&mut self) -> Result<(), WorkloadError> {
        self.state = SinkState::Completed;
        self.timing.mark("complete");
        Ok(())
    }

    /// Get the list of sections sent.
    pub fn sections_sent(&self) -> &[String] {
        &self.sections_sent
    }

    /// Get the current lifecycle phase.
    pub fn phase(&self) -> LifecyclePhase {
        match self.state {
            SinkState::Initial => LifecyclePhase::Start,
            SinkState::ShellSent if self.sections_sent.is_empty() => LifecyclePhase::ShellSent,
            SinkState::ShellSent => {
                LifecyclePhase::SectionSent(self.sections_sent.last().unwrap().clone())
            }
            SinkState::Completed => LifecyclePhase::Completion,
        }
    }

    /// Get timing context reference.
    pub fn timing(&self) -> &TimingContext {
        &self.timing
    }

    /// Get mutable access to the underlying sink for advanced use.
    pub fn inner_mut(&mut self) -> &mut S {
        &mut self.inner
    }

    /// Consume the sink and return the inner value.
    pub fn into_inner(self) -> S {
        self.inner
    }
}
