//! Dependency tagging for semantic categorization.

use std::time::Duration;

/// Well-known dependency categories with semantic meaning.
///
/// Each tag carries default timeouts, retry policies, and concurrency limits.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DependencyTag {
    /// Search API calls.
    Search,
    /// Pricing/catalog API calls.
    Pricing,
    /// User profile/personalization.
    Profile,
    /// Content Management System.
    Cms,
    /// Product recommendations.
    Recommendations,
    /// Inventory/availability checks.
    Inventory,
    /// Reviews and ratings.
    Reviews,
    /// Advertising/sponsored content.
    Ads,
    /// Analytics/tracking.
    Analytics,
    /// Custom dependency with name.
    Custom(&'static str),
}

impl DependencyTag {
    /// Get the default timeout for this dependency type.
    pub fn default_timeout(&self) -> Duration {
        match self {
            Self::Search => Duration::from_millis(500),
            Self::Pricing => Duration::from_millis(200),
            Self::Profile => Duration::from_millis(300),
            Self::Cms => Duration::from_millis(1000),
            Self::Recommendations => Duration::from_millis(400),
            Self::Inventory => Duration::from_millis(150),
            Self::Reviews => Duration::from_millis(500),
            Self::Ads => Duration::from_millis(200),
            Self::Analytics => Duration::from_millis(100),
            Self::Custom(_) => Duration::from_millis(500),
        }
    }

    /// Get the default max retries for this dependency type.
    pub fn default_max_retries(&self) -> u32 {
        match self {
            Self::Pricing | Self::Inventory => 2, // Critical, retry more
            Self::Analytics | Self::Ads => 0,     // Non-critical, don't retry
            _ => 1,
        }
    }

    /// Get the default concurrency limit for this dependency type.
    pub fn default_concurrency(&self) -> usize {
        match self {
            Self::Search => 2,
            Self::Analytics => 5,
            _ => 3,
        }
    }

    /// Check if this dependency is critical (should block render).
    pub fn is_critical(&self) -> bool {
        matches!(self, Self::Pricing | Self::Inventory | Self::Search)
    }

    /// Get the name of this dependency.
    pub fn name(&self) -> &str {
        match self {
            Self::Search => "search",
            Self::Pricing => "pricing",
            Self::Profile => "profile",
            Self::Cms => "cms",
            Self::Recommendations => "recommendations",
            Self::Inventory => "inventory",
            Self::Reviews => "reviews",
            Self::Ads => "ads",
            Self::Analytics => "analytics",
            Self::Custom(name) => name,
        }
    }
}

impl std::fmt::Display for DependencyTag {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name())
    }
}
