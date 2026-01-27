//! Section scheduler for concurrent execution.

use std::collections::{HashMap, HashSet};

use edge_streaming::Section;

/// Status of a section in the scheduler.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SectionStatus {
    /// Waiting for dependencies.
    Pending,
    /// Currently being rendered.
    InProgress,
    /// Successfully completed.
    Completed,
    /// Failed with error.
    Failed(String),
    /// Skipped (e.g., non-critical failure).
    Skipped,
}

/// A section being tracked by the scheduler.
#[derive(Debug)]
pub struct ScheduledSection {
    /// The section definition.
    pub section: Section,
    /// Current status.
    pub status: SectionStatus,
}

/// Schedules sections based on dependency readiness.
///
/// Enables concurrent execution where sections stream as their
/// dependencies complete, without blocking on slower sections.
#[derive(Debug)]
pub struct SectionScheduler {
    /// Sections to schedule.
    sections: HashMap<String, ScheduledSection>,
    /// Order sections were added (for fallback ordering).
    order: Vec<String>,
    /// Completed dependencies.
    completed_deps: HashSet<String>,
}

impl SectionScheduler {
    /// Create a new scheduler.
    pub fn new() -> Self {
        Self {
            sections: HashMap::new(),
            order: Vec::new(),
            completed_deps: HashSet::new(),
        }
    }

    /// Add a section to the scheduler.
    pub fn add_section(&mut self, section: Section) {
        let name = section.name.clone();
        self.sections.insert(
            name.clone(),
            ScheduledSection {
                section,
                status: SectionStatus::Pending,
            },
        );
        self.order.push(name);
    }

    /// Mark a dependency as completed.
    pub fn complete_dependency(&mut self, dep: &str) {
        self.completed_deps.insert(dep.to_string());
    }

    /// Get sections that are ready to execute (all dependencies met).
    pub fn ready_sections(&self) -> Vec<&Section> {
        self.sections
            .values()
            .filter(|s| {
                s.status == SectionStatus::Pending
                    && s.section
                        .dependencies
                        .iter()
                        .all(|d| self.completed_deps.contains(d))
            })
            .map(|s| &s.section)
            .collect()
    }

    /// Mark a section as in progress.
    pub fn start_section(&mut self, name: &str) {
        if let Some(s) = self.sections.get_mut(name) {
            s.status = SectionStatus::InProgress;
        }
    }

    /// Mark a section as completed.
    pub fn complete_section(&mut self, name: &str) {
        if let Some(s) = self.sections.get_mut(name) {
            s.status = SectionStatus::Completed;
        }
    }

    /// Mark a section as failed.
    pub fn fail_section(&mut self, name: &str, error: impl Into<String>) {
        if let Some(s) = self.sections.get_mut(name) {
            s.status = SectionStatus::Failed(error.into());
        }
    }

    /// Get all pending sections.
    pub fn pending_sections(&self) -> Vec<&Section> {
        self.sections
            .values()
            .filter(|s| s.status == SectionStatus::Pending)
            .map(|s| &s.section)
            .collect()
    }

    /// Check if all sections are complete (or failed/skipped).
    pub fn is_complete(&self) -> bool {
        self.sections.values().all(|s| {
            matches!(
                s.status,
                SectionStatus::Completed | SectionStatus::Failed(_) | SectionStatus::Skipped
            )
        })
    }

    /// Get section status by name.
    pub fn status(&self, name: &str) -> Option<&SectionStatus> {
        self.sections.get(name).map(|s| &s.status)
    }
}

impl Default for SectionScheduler {
    fn default() -> Self {
        Self::new()
    }
}
