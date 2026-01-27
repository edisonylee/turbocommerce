//! Structured logging with request context.

use std::collections::HashMap;
use std::fmt;

use edge_core::RequestId;
use serde::Serialize;

/// Log level for structured logs.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum LogLevel {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
}

impl fmt::Display for LogLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Trace => write!(f, "TRACE"),
            Self::Debug => write!(f, "DEBUG"),
            Self::Info => write!(f, "INFO"),
            Self::Warn => write!(f, "WARN"),
            Self::Error => write!(f, "ERROR"),
        }
    }
}

/// A structured log entry.
#[derive(Debug, Clone, Serialize)]
pub struct LogEntry {
    /// Log level.
    pub level: LogLevel,
    /// Log message.
    pub message: String,
    /// Request ID for correlation.
    pub request_id: String,
    /// Workload name.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub workload: Option<String>,
    /// Route path.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub route: Option<String>,
    /// Additional structured fields.
    #[serde(flatten)]
    pub fields: HashMap<String, serde_json::Value>,
    /// Timestamp in microseconds since request start.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub elapsed_us: Option<u64>,
}

impl LogEntry {
    /// Format as JSON string.
    pub fn to_json(&self) -> String {
        serde_json::to_string(self).unwrap_or_else(|_| self.message.clone())
    }

    /// Format as human-readable string.
    pub fn to_human(&self) -> String {
        let mut s = format!("[{}] {}", self.level, self.message);

        if let Some(elapsed) = self.elapsed_us {
            s.push_str(&format!(" ({}us)", elapsed));
        }

        if !self.fields.is_empty() {
            s.push_str(" | ");
            let fields: Vec<String> = self
                .fields
                .iter()
                .map(|(k, v)| format!("{}={}", k, v))
                .collect();
            s.push_str(&fields.join(" "));
        }

        s
    }
}

/// Structured logger with request context.
///
/// Provides structured logging with automatic request ID propagation
/// and timing information.
#[derive(Debug, Clone)]
pub struct StructuredLogger {
    request_id: RequestId,
    workload: Option<String>,
    route: Option<String>,
    start_time: std::time::Instant,
    min_level: LogLevel,
    format: LogFormat,
}

/// Output format for logs.
#[derive(Debug, Clone, Copy, Default)]
pub enum LogFormat {
    /// JSON format (for production/log aggregation).
    #[default]
    Json,
    /// Human-readable format (for development).
    Human,
}

impl StructuredLogger {
    /// Create a new logger with request context.
    pub fn new(request_id: RequestId) -> Self {
        Self {
            request_id,
            workload: None,
            route: None,
            start_time: std::time::Instant::now(),
            min_level: LogLevel::Info,
            format: LogFormat::Json,
        }
    }

    /// Set the workload name.
    pub fn with_workload(mut self, workload: impl Into<String>) -> Self {
        self.workload = Some(workload.into());
        self
    }

    /// Set the route path.
    pub fn with_route(mut self, route: impl Into<String>) -> Self {
        self.route = Some(route.into());
        self
    }

    /// Set minimum log level.
    pub fn with_min_level(mut self, level: LogLevel) -> Self {
        self.min_level = level;
        self
    }

    /// Set output format.
    pub fn with_format(mut self, format: LogFormat) -> Self {
        self.format = format;
        self
    }

    /// Log at trace level.
    pub fn trace(&self, message: &str) {
        self.log(LogLevel::Trace, message, HashMap::new());
    }

    /// Log at debug level.
    pub fn debug(&self, message: &str) {
        self.log(LogLevel::Debug, message, HashMap::new());
    }

    /// Log at info level.
    pub fn info(&self, message: &str) {
        self.log(LogLevel::Info, message, HashMap::new());
    }

    /// Log at warn level.
    pub fn warn(&self, message: &str) {
        self.log(LogLevel::Warn, message, HashMap::new());
    }

    /// Log at error level.
    pub fn error(&self, message: &str) {
        self.log(LogLevel::Error, message, HashMap::new());
    }

    /// Log with additional fields.
    pub fn log_with_fields(
        &self,
        level: LogLevel,
        message: &str,
        fields: HashMap<String, serde_json::Value>,
    ) {
        self.log(level, message, fields);
    }

    /// Log at info level with fields.
    pub fn info_with(&self, message: &str, fields: &[(&str, &dyn fmt::Debug)]) {
        let fields = fields
            .iter()
            .map(|(k, v)| (k.to_string(), serde_json::json!(format!("{:?}", v))))
            .collect();
        self.log(LogLevel::Info, message, fields);
    }

    /// Log at error level with fields.
    pub fn error_with(&self, message: &str, fields: &[(&str, &dyn fmt::Debug)]) {
        let fields = fields
            .iter()
            .map(|(k, v)| (k.to_string(), serde_json::json!(format!("{:?}", v))))
            .collect();
        self.log(LogLevel::Error, message, fields);
    }

    fn log(&self, level: LogLevel, message: &str, fields: HashMap<String, serde_json::Value>) {
        if level < self.min_level {
            return;
        }

        let entry = LogEntry {
            level,
            message: message.to_string(),
            request_id: self.request_id.to_string(),
            workload: self.workload.clone(),
            route: self.route.clone(),
            fields,
            elapsed_us: Some(self.start_time.elapsed().as_micros() as u64),
        };

        let output = match self.format {
            LogFormat::Json => entry.to_json(),
            LogFormat::Human => entry.to_human(),
        };

        // Output to stderr (Spin captures this)
        eprintln!("{}", output);
    }

    /// Get the request ID.
    pub fn request_id(&self) -> &RequestId {
        &self.request_id
    }

    /// Get elapsed time since logger creation.
    pub fn elapsed_us(&self) -> u64 {
        self.start_time.elapsed().as_micros() as u64
    }
}

/// Builder for log entries with fluent API.
pub struct LogBuilder<'a> {
    logger: &'a StructuredLogger,
    level: LogLevel,
    message: String,
    fields: HashMap<String, serde_json::Value>,
}

impl<'a> LogBuilder<'a> {
    /// Create a new log builder.
    pub fn new(logger: &'a StructuredLogger, level: LogLevel, message: impl Into<String>) -> Self {
        Self {
            logger,
            level,
            message: message.into(),
            fields: HashMap::new(),
        }
    }

    /// Add a string field.
    pub fn field(mut self, key: &str, value: impl Into<String>) -> Self {
        self.fields
            .insert(key.to_string(), serde_json::json!(value.into()));
        self
    }

    /// Add an integer field.
    pub fn field_i64(mut self, key: &str, value: i64) -> Self {
        self.fields.insert(key.to_string(), serde_json::json!(value));
        self
    }

    /// Add a float field.
    pub fn field_f64(mut self, key: &str, value: f64) -> Self {
        self.fields.insert(key.to_string(), serde_json::json!(value));
        self
    }

    /// Add a boolean field.
    pub fn field_bool(mut self, key: &str, value: bool) -> Self {
        self.fields.insert(key.to_string(), serde_json::json!(value));
        self
    }

    /// Add a duration field (in milliseconds).
    pub fn duration_ms(mut self, key: &str, duration: std::time::Duration) -> Self {
        self.fields
            .insert(key.to_string(), serde_json::json!(duration.as_millis()));
        self
    }

    /// Emit the log entry.
    pub fn emit(self) {
        self.logger.log(self.level, &self.message, self.fields);
    }
}

impl StructuredLogger {
    /// Start building an info log entry.
    pub fn info_builder(&self, message: impl Into<String>) -> LogBuilder<'_> {
        LogBuilder::new(self, LogLevel::Info, message)
    }

    /// Start building a warn log entry.
    pub fn warn_builder(&self, message: impl Into<String>) -> LogBuilder<'_> {
        LogBuilder::new(self, LogLevel::Warn, message)
    }

    /// Start building an error log entry.
    pub fn error_builder(&self, message: impl Into<String>) -> LogBuilder<'_> {
        LogBuilder::new(self, LogLevel::Error, message)
    }

    /// Start building a debug log entry.
    pub fn debug_builder(&self, message: impl Into<String>) -> LogBuilder<'_> {
        LogBuilder::new(self, LogLevel::Debug, message)
    }
}
