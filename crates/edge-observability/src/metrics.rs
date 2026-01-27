//! Platform-level timing metrics.

use std::collections::HashMap;
use std::time::{Duration, Instant};

use edge_core::RequestId;
use serde::Serialize;

/// Platform metrics for a single request.
#[derive(Debug, Clone, Serialize, serde::Deserialize)]
pub struct RequestMetrics {
    /// Request ID for correlation.
    pub request_id: String,
    /// Workload name.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub workload: Option<String>,
    /// Route path.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub route: Option<String>,
    /// Time to shell flush (microseconds).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub time_to_shell_us: Option<u64>,
    /// Time to first section (microseconds).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub time_to_first_section_us: Option<u64>,
    /// Time to full page (microseconds).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub time_to_full_page_us: Option<u64>,
    /// Section timings.
    pub sections: HashMap<String, SectionMetrics>,
    /// Dependency timings.
    pub dependencies: HashMap<String, DependencyMetrics>,
    /// Total request duration (microseconds).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total_duration_us: Option<u64>,
    /// HTTP status code.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status_code: Option<u16>,
}

/// Metrics for a single section.
#[derive(Debug, Clone, Serialize, serde::Deserialize)]
pub struct SectionMetrics {
    /// Section name.
    pub name: String,
    /// Time from request start to section start (microseconds).
    pub start_us: u64,
    /// Time from request start to section sent (microseconds).
    pub sent_us: u64,
    /// Section render duration (microseconds).
    pub duration_us: u64,
    /// Bytes sent for this section.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bytes: Option<usize>,
    /// Whether section used fallback.
    pub used_fallback: bool,
}

/// Metrics for a dependency fetch.
#[derive(Debug, Clone, Serialize, serde::Deserialize)]
pub struct DependencyMetrics {
    /// Dependency tag/name.
    pub tag: String,
    /// URL fetched.
    pub url: String,
    /// Fetch duration (microseconds).
    pub duration_us: u64,
    /// HTTP status code.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status_code: Option<u16>,
    /// Response size in bytes.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub response_bytes: Option<usize>,
    /// Whether the request was retried.
    pub retried: bool,
    /// Number of retry attempts.
    pub retry_count: u32,
    /// Whether the request succeeded.
    pub success: bool,
    /// Error message if failed.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

/// Collector for request metrics.
#[derive(Debug)]
pub struct MetricsCollector {
    request_id: RequestId,
    workload: Option<String>,
    route: Option<String>,
    start: Instant,
    shell_sent: Option<Instant>,
    first_section_sent: Option<Instant>,
    sections: HashMap<String, SectionMetricsBuilder>,
    dependencies: HashMap<String, DependencyMetrics>,
}

#[derive(Debug)]
struct SectionMetricsBuilder {
    name: String,
    start: Option<Instant>,
    sent: Option<Instant>,
    bytes: Option<usize>,
    used_fallback: bool,
}

impl MetricsCollector {
    /// Create a new metrics collector.
    pub fn new(request_id: RequestId) -> Self {
        Self {
            request_id,
            workload: None,
            route: None,
            start: Instant::now(),
            shell_sent: None,
            first_section_sent: None,
            sections: HashMap::new(),
            dependencies: HashMap::new(),
        }
    }

    /// Set workload name.
    pub fn set_workload(&mut self, workload: impl Into<String>) {
        self.workload = Some(workload.into());
    }

    /// Set route path.
    pub fn set_route(&mut self, route: impl Into<String>) {
        self.route = Some(route.into());
    }

    /// Record shell sent.
    pub fn record_shell_sent(&mut self) {
        self.shell_sent = Some(Instant::now());
    }

    /// Record section start.
    pub fn record_section_start(&mut self, name: &str) {
        self.sections.insert(
            name.to_string(),
            SectionMetricsBuilder {
                name: name.to_string(),
                start: Some(Instant::now()),
                sent: None,
                bytes: None,
                used_fallback: false,
            },
        );
    }

    /// Record section sent.
    pub fn record_section_sent(&mut self, name: &str, bytes: Option<usize>, used_fallback: bool) {
        let now = Instant::now();

        if self.first_section_sent.is_none() {
            self.first_section_sent = Some(now);
        }

        if let Some(section) = self.sections.get_mut(name) {
            section.sent = Some(now);
            section.bytes = bytes;
            section.used_fallback = used_fallback;
        } else {
            // Section wasn't started explicitly, record it now
            self.sections.insert(
                name.to_string(),
                SectionMetricsBuilder {
                    name: name.to_string(),
                    start: Some(now),
                    sent: Some(now),
                    bytes,
                    used_fallback,
                },
            );
        }
    }

    /// Record a dependency fetch.
    pub fn record_dependency(
        &mut self,
        tag: &str,
        url: &str,
        duration: Duration,
        status_code: Option<u16>,
        response_bytes: Option<usize>,
        retried: bool,
        retry_count: u32,
        success: bool,
        error: Option<String>,
    ) {
        let key = format!("{}:{}", tag, url);
        self.dependencies.insert(
            key,
            DependencyMetrics {
                tag: tag.to_string(),
                url: url.to_string(),
                duration_us: duration.as_micros() as u64,
                status_code,
                response_bytes,
                retried,
                retry_count,
                success,
                error,
            },
        );
    }

    /// Finalize and return the metrics.
    pub fn finalize(self, status_code: Option<u16>) -> RequestMetrics {
        let now = Instant::now();
        let start = self.start;

        let time_to_shell_us = self
            .shell_sent
            .map(|t| t.duration_since(start).as_micros() as u64);

        let time_to_first_section_us = self
            .first_section_sent
            .map(|t| t.duration_since(start).as_micros() as u64);

        let sections: HashMap<String, SectionMetrics> = self
            .sections
            .into_iter()
            .filter_map(|(name, builder)| {
                let start = builder.start?;
                let sent = builder.sent.unwrap_or(now);
                Some((
                    name.clone(),
                    SectionMetrics {
                        name,
                        start_us: start.duration_since(self.start).as_micros() as u64,
                        sent_us: sent.duration_since(self.start).as_micros() as u64,
                        duration_us: sent.duration_since(start).as_micros() as u64,
                        bytes: builder.bytes,
                        used_fallback: builder.used_fallback,
                    },
                ))
            })
            .collect();

        RequestMetrics {
            request_id: self.request_id.to_string(),
            workload: self.workload,
            route: self.route,
            time_to_shell_us,
            time_to_first_section_us,
            time_to_full_page_us: Some(now.duration_since(start).as_micros() as u64),
            sections,
            dependencies: self.dependencies,
            total_duration_us: Some(now.duration_since(start).as_micros() as u64),
            status_code,
        }
    }

    /// Get time-to-shell so far.
    pub fn time_to_shell(&self) -> Option<Duration> {
        self.shell_sent.map(|t| t.duration_since(self.start))
    }

    /// Get time-to-first-section so far.
    pub fn time_to_first_section(&self) -> Option<Duration> {
        self.first_section_sent
            .map(|t| t.duration_since(self.start))
    }

    /// Get total elapsed time.
    pub fn elapsed(&self) -> Duration {
        self.start.elapsed()
    }
}

impl RequestMetrics {
    /// Format as JSON.
    pub fn to_json(&self) -> String {
        serde_json::to_string(self).unwrap_or_default()
    }

    /// Format as JSON (pretty printed).
    pub fn to_json_pretty(&self) -> String {
        serde_json::to_string_pretty(self).unwrap_or_default()
    }

    /// Format as human-readable summary.
    pub fn to_summary(&self) -> String {
        let mut lines = Vec::new();

        lines.push(format!("Request: {}", self.request_id));

        if let Some(tts) = self.time_to_shell_us {
            lines.push(format!("  Time to shell: {}us ({:.2}ms)", tts, tts as f64 / 1000.0));
        }

        if let Some(ttfs) = self.time_to_first_section_us {
            lines.push(format!(
                "  Time to first section: {}us ({:.2}ms)",
                ttfs,
                ttfs as f64 / 1000.0
            ));
        }

        if let Some(ttfp) = self.time_to_full_page_us {
            lines.push(format!(
                "  Time to full page: {}us ({:.2}ms)",
                ttfp,
                ttfp as f64 / 1000.0
            ));
        }

        if !self.sections.is_empty() {
            lines.push("  Sections:".to_string());
            for (name, section) in &self.sections {
                let fallback = if section.used_fallback { " [fallback]" } else { "" };
                lines.push(format!(
                    "    {}: {}us ({:.2}ms){}",
                    name,
                    section.duration_us,
                    section.duration_us as f64 / 1000.0,
                    fallback
                ));
            }
        }

        if !self.dependencies.is_empty() {
            lines.push("  Dependencies:".to_string());
            for dep in self.dependencies.values() {
                let status = if dep.success {
                    format!("{}", dep.status_code.unwrap_or(0))
                } else {
                    "FAILED".to_string()
                };
                lines.push(format!(
                    "    {} [{}]: {}us ({:.2}ms) - {}",
                    dep.tag,
                    status,
                    dep.duration_us,
                    dep.duration_us as f64 / 1000.0,
                    dep.url
                ));
            }
        }

        lines.join("\n")
    }
}
