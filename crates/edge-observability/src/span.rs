//! Trace context and span management.

use edge_core::RequestId;

/// Trace context for distributed tracing.
///
/// Compatible with W3C Trace Context format.
#[derive(Debug, Clone)]
pub struct TraceContext {
    /// The trace ID (128-bit, hex encoded).
    pub trace_id: String,
    /// The span ID (64-bit, hex encoded).
    pub span_id: String,
    /// Parent span ID if this is a child span.
    pub parent_span_id: Option<String>,
    /// Trace flags (e.g., sampled).
    pub flags: TraceFlags,
}

/// Trace flags indicating sampling decisions.
#[derive(Debug, Clone, Copy, Default)]
pub struct TraceFlags {
    /// Whether this trace is sampled.
    pub sampled: bool,
}

impl TraceContext {
    /// Create a new root trace context.
    pub fn new() -> Self {
        Self {
            trace_id: generate_trace_id(),
            span_id: generate_span_id(),
            parent_span_id: None,
            flags: TraceFlags { sampled: true },
        }
    }

    /// Create a child span from this context.
    pub fn child(&self) -> Self {
        Self {
            trace_id: self.trace_id.clone(),
            span_id: generate_span_id(),
            parent_span_id: Some(self.span_id.clone()),
            flags: self.flags,
        }
    }

    /// Create from a request ID (uses request ID as trace ID).
    pub fn from_request_id(request_id: &RequestId) -> Self {
        Self {
            trace_id: request_id.0.clone(),
            span_id: generate_span_id(),
            parent_span_id: None,
            flags: TraceFlags { sampled: true },
        }
    }

    /// Parse from W3C traceparent header.
    ///
    /// Format: `{version}-{trace_id}-{span_id}-{flags}`
    /// Example: `00-0af7651916cd43dd8448eb211c80319c-b7ad6b7169203331-01`
    pub fn from_traceparent(header: &str) -> Option<Self> {
        let parts: Vec<&str> = header.split('-').collect();
        if parts.len() != 4 {
            return None;
        }

        let version = parts[0];
        if version != "00" {
            return None; // Only support version 00
        }

        let trace_id = parts[1].to_string();
        let span_id = parts[2].to_string();
        let flags = u8::from_str_radix(parts[3], 16).unwrap_or(0);

        Some(Self {
            trace_id,
            span_id,
            parent_span_id: None,
            flags: TraceFlags {
                sampled: flags & 0x01 != 0,
            },
        })
    }

    /// Format as W3C traceparent header.
    pub fn to_traceparent(&self) -> String {
        let flags = if self.flags.sampled { "01" } else { "00" };
        format!("00-{}-{}-{}", self.trace_id, self.span_id, flags)
    }
}

impl Default for TraceContext {
    fn default() -> Self {
        Self::new()
    }
}

/// A span representing a unit of work.
#[derive(Debug, Clone)]
pub struct Span {
    /// Span name (e.g., "fetch_todos", "render_section").
    pub name: String,
    /// Trace context.
    pub context: TraceContext,
    /// Start time in microseconds since request start.
    pub start_us: u64,
    /// End time in microseconds (None if still open).
    pub end_us: Option<u64>,
    /// Span attributes.
    pub attributes: Vec<(String, SpanValue)>,
    /// Span status.
    pub status: SpanStatus,
}

/// Span attribute value types.
#[derive(Debug, Clone)]
pub enum SpanValue {
    String(String),
    Int(i64),
    Float(f64),
    Bool(bool),
}

impl From<&str> for SpanValue {
    fn from(s: &str) -> Self {
        Self::String(s.to_string())
    }
}

impl From<String> for SpanValue {
    fn from(s: String) -> Self {
        Self::String(s)
    }
}

impl From<i64> for SpanValue {
    fn from(v: i64) -> Self {
        Self::Int(v)
    }
}

impl From<f64> for SpanValue {
    fn from(v: f64) -> Self {
        Self::Float(v)
    }
}

impl From<bool> for SpanValue {
    fn from(v: bool) -> Self {
        Self::Bool(v)
    }
}

/// Span status codes.
#[derive(Debug, Clone, Copy, Default)]
pub enum SpanStatus {
    #[default]
    Unset,
    Ok,
    Error,
}

impl Span {
    /// Create a new span.
    pub fn new(name: impl Into<String>, context: TraceContext, start_us: u64) -> Self {
        Self {
            name: name.into(),
            context,
            start_us,
            end_us: None,
            attributes: Vec::new(),
            status: SpanStatus::Unset,
        }
    }

    /// Add an attribute to the span.
    pub fn set_attribute(&mut self, key: impl Into<String>, value: impl Into<SpanValue>) {
        self.attributes.push((key.into(), value.into()));
    }

    /// Mark the span as complete.
    pub fn end(&mut self, end_us: u64) {
        self.end_us = Some(end_us);
    }

    /// Set span status to OK.
    pub fn set_ok(&mut self) {
        self.status = SpanStatus::Ok;
    }

    /// Set span status to Error.
    pub fn set_error(&mut self) {
        self.status = SpanStatus::Error;
    }

    /// Get duration in microseconds (None if not ended).
    pub fn duration_us(&self) -> Option<u64> {
        self.end_us.map(|end| end.saturating_sub(self.start_us))
    }
}

// Simple ID generation (not cryptographically secure, but good enough for tracing)
fn generate_trace_id() -> String {
    format!(
        "{:016x}{:016x}",
        simple_random_u64(),
        simple_random_u64()
    )
}

fn generate_span_id() -> String {
    format!("{:016x}", simple_random_u64())
}

fn simple_random_u64() -> u64 {
    use std::time::{SystemTime, UNIX_EPOCH};

    static mut COUNTER: u64 = 0;

    let time = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos() as u64;

    unsafe {
        COUNTER = COUNTER.wrapping_add(1);
        time ^ (COUNTER.wrapping_mul(0x517cc1b727220a95))
    }
}
