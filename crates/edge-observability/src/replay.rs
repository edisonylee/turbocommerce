//! Request replay for local debugging.

use std::collections::HashMap;

use edge_core::RequestId;
use serde::{Deserialize, Serialize};

use crate::metrics::RequestMetrics;

/// A recorded request for replay.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecordedRequest {
    /// Request ID.
    pub request_id: String,
    /// HTTP method.
    pub method: String,
    /// Request path.
    pub path: String,
    /// Query string.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub query: Option<String>,
    /// Request headers.
    pub headers: HashMap<String, String>,
    /// Request body (if any).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub body: Option<Vec<u8>>,
}

/// A recorded dependency response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecordedDependency {
    /// Dependency tag.
    pub tag: String,
    /// URL fetched.
    pub url: String,
    /// HTTP status code.
    pub status_code: u16,
    /// Response headers.
    pub headers: HashMap<String, String>,
    /// Response body.
    pub body: Vec<u8>,
    /// Fetch duration in microseconds.
    pub duration_us: u64,
}

/// A recorded section.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecordedSection {
    /// Section name.
    pub name: String,
    /// Section HTML content.
    pub content: String,
    /// Time from request start to section sent (microseconds).
    pub sent_at_us: u64,
}

/// A complete recording of a request/response cycle.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Recording {
    /// Recording version.
    pub version: u32,
    /// Timestamp when recorded.
    pub timestamp: String,
    /// The original request.
    pub request: RecordedRequest,
    /// Recorded dependency responses.
    pub dependencies: Vec<RecordedDependency>,
    /// Recorded sections.
    pub sections: Vec<RecordedSection>,
    /// Final response status code.
    pub response_status: u16,
    /// Final response headers.
    pub response_headers: HashMap<String, String>,
    /// Request metrics.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metrics: Option<RequestMetrics>,
}

impl Recording {
    /// Current recording format version.
    pub const VERSION: u32 = 1;

    /// Serialize to JSON.
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }

    /// Deserialize from JSON.
    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }
}

/// Records request/response data for replay.
#[derive(Debug)]
pub struct ReplayRecorder {
    request: RecordedRequest,
    dependencies: Vec<RecordedDependency>,
    sections: Vec<RecordedSection>,
    start_time: std::time::Instant,
}

impl ReplayRecorder {
    /// Create a new recorder for a request.
    pub fn new(
        request_id: RequestId,
        method: &str,
        path: &str,
        query: Option<&str>,
        headers: HashMap<String, String>,
        body: Option<Vec<u8>>,
    ) -> Self {
        Self {
            request: RecordedRequest {
                request_id: request_id.to_string(),
                method: method.to_string(),
                path: path.to_string(),
                query: query.map(|s| s.to_string()),
                headers,
                body,
            },
            dependencies: Vec::new(),
            sections: Vec::new(),
            start_time: std::time::Instant::now(),
        }
    }

    /// Record a dependency response.
    pub fn record_dependency(
        &mut self,
        tag: &str,
        url: &str,
        status_code: u16,
        headers: HashMap<String, String>,
        body: Vec<u8>,
        duration: std::time::Duration,
    ) {
        self.dependencies.push(RecordedDependency {
            tag: tag.to_string(),
            url: url.to_string(),
            status_code,
            headers,
            body,
            duration_us: duration.as_micros() as u64,
        });
    }

    /// Record a section.
    pub fn record_section(&mut self, name: &str, content: &str) {
        self.sections.push(RecordedSection {
            name: name.to_string(),
            content: content.to_string(),
            sent_at_us: self.start_time.elapsed().as_micros() as u64,
        });
    }

    /// Finalize the recording.
    pub fn finalize(
        self,
        response_status: u16,
        response_headers: HashMap<String, String>,
        metrics: Option<RequestMetrics>,
    ) -> Recording {
        Recording {
            version: Recording::VERSION,
            timestamp: chrono_lite_now(),
            request: self.request,
            dependencies: self.dependencies,
            sections: self.sections,
            response_status,
            response_headers,
            metrics,
        }
    }
}

/// Replays recorded requests for debugging.
#[derive(Debug)]
pub struct ReplayPlayer {
    recording: Recording,
}

impl ReplayPlayer {
    /// Load a recording.
    pub fn new(recording: Recording) -> Self {
        Self { recording }
    }

    /// Load from JSON.
    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        Ok(Self::new(Recording::from_json(json)?))
    }

    /// Get the request to replay.
    pub fn request(&self) -> &RecordedRequest {
        &self.recording.request
    }

    /// Get recorded dependency by tag and URL.
    pub fn get_dependency(&self, tag: &str, url: &str) -> Option<&RecordedDependency> {
        self.recording
            .dependencies
            .iter()
            .find(|d| d.tag == tag && d.url == url)
    }

    /// Get all recorded dependencies.
    pub fn dependencies(&self) -> &[RecordedDependency] {
        &self.recording.dependencies
    }

    /// Get recorded sections.
    pub fn sections(&self) -> &[RecordedSection] {
        &self.recording.sections
    }

    /// Get original metrics.
    pub fn metrics(&self) -> Option<&RequestMetrics> {
        self.recording.metrics.as_ref()
    }

    /// Get original response status.
    pub fn response_status(&self) -> u16 {
        self.recording.response_status
    }
}

/// Result of comparing two recordings.
#[derive(Debug, Serialize)]
pub struct ReplayDiff {
    /// Whether the recordings match.
    pub matches: bool,
    /// Section differences.
    pub section_diffs: Vec<SectionDiff>,
    /// Metric differences.
    pub metric_diffs: Vec<MetricDiff>,
}

/// Difference in a section.
#[derive(Debug, Serialize)]
pub struct SectionDiff {
    /// Section name.
    pub section: String,
    /// Type of difference.
    pub diff_type: DiffType,
    /// Expected content (from recording).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expected: Option<String>,
    /// Actual content (from replay).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub actual: Option<String>,
}

/// Difference in a metric.
#[derive(Debug, Serialize)]
pub struct MetricDiff {
    /// Metric name.
    pub metric: String,
    /// Expected value.
    pub expected: String,
    /// Actual value.
    pub actual: String,
    /// Percentage difference (for numeric values).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub percent_diff: Option<f64>,
}

/// Type of difference.
#[derive(Debug, Clone, Copy, Serialize)]
pub enum DiffType {
    /// Section missing in actual.
    Missing,
    /// Section added in actual.
    Added,
    /// Content differs.
    ContentMismatch,
}

impl ReplayDiff {
    /// Compare two sets of sections.
    pub fn compare_sections(
        expected: &[RecordedSection],
        actual: &[RecordedSection],
    ) -> Vec<SectionDiff> {
        let mut diffs = Vec::new();

        // Check for missing sections
        for exp in expected {
            if !actual.iter().any(|a| a.name == exp.name) {
                diffs.push(SectionDiff {
                    section: exp.name.clone(),
                    diff_type: DiffType::Missing,
                    expected: Some(exp.content.clone()),
                    actual: None,
                });
            }
        }

        // Check for added sections
        for act in actual {
            if !expected.iter().any(|e| e.name == act.name) {
                diffs.push(SectionDiff {
                    section: act.name.clone(),
                    diff_type: DiffType::Added,
                    expected: None,
                    actual: Some(act.content.clone()),
                });
            }
        }

        // Check for content mismatches
        for exp in expected {
            if let Some(act) = actual.iter().find(|a| a.name == exp.name) {
                if exp.content != act.content {
                    diffs.push(SectionDiff {
                        section: exp.name.clone(),
                        diff_type: DiffType::ContentMismatch,
                        expected: Some(exp.content.clone()),
                        actual: Some(act.content.clone()),
                    });
                }
            }
        }

        diffs
    }
}

// Simple timestamp without full chrono dependency
fn chrono_lite_now() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};

    let duration = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();

    let secs = duration.as_secs();
    let _millis = duration.subsec_millis();

    // Simple ISO-ish format
    format!("{}", secs)
}
