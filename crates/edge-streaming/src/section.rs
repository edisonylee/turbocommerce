//! Section abstraction for independently streamable page parts.

use std::time::Duration;

/// A section is a named, independently-streamable part of the page.
#[derive(Debug, Clone)]
pub struct Section {
    /// Section name (used for timing and identification).
    pub name: String,
    /// Dependencies this section requires.
    pub dependencies: Vec<String>,
    /// Fallback HTML if dependencies fail.
    pub fallback: Option<String>,
    /// Timeout for this section's dependencies.
    pub timeout: Option<Duration>,
}

impl Section {
    /// Create a new section.
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            dependencies: Vec::new(),
            fallback: None,
            timeout: None,
        }
    }

    /// Create a section using the builder.
    pub fn builder(name: impl Into<String>) -> SectionBuilder {
        SectionBuilder::new(name)
    }
}

/// Builder for ergonomic section definition.
pub struct SectionBuilder {
    name: String,
    dependencies: Vec<String>,
    fallback: Option<String>,
    timeout: Option<Duration>,
}

impl SectionBuilder {
    /// Create a new section builder.
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            dependencies: Vec::new(),
            fallback: None,
            timeout: None,
        }
    }

    /// Add a dependency tag.
    pub fn depends_on(mut self, tag: impl Into<String>) -> Self {
        self.dependencies.push(tag.into());
        self
    }

    /// Add multiple dependencies.
    pub fn depends_on_all(mut self, tags: &[&str]) -> Self {
        self.dependencies.extend(tags.iter().map(|s| s.to_string()));
        self
    }

    /// Set fallback HTML.
    pub fn with_fallback(mut self, html: impl Into<String>) -> Self {
        self.fallback = Some(html.into());
        self
    }

    /// Set timeout for this section.
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = Some(timeout);
        self
    }

    /// Build the section.
    pub fn build(self) -> Section {
        Section {
            name: self.name,
            dependencies: self.dependencies,
            fallback: self.fallback,
            timeout: self.timeout,
        }
    }
}
