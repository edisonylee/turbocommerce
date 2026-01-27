//! TurboApp configuration and setup.

use turbo_router::RouteRegistry;

/// Configuration for a TurboCommerce application.
#[derive(Debug, Clone)]
pub struct TurboConfig {
    /// Application name.
    pub name: String,
    /// Whether streaming SSR is enabled.
    pub streaming: bool,
    /// Default page title.
    pub default_title: String,
    /// CSS file path.
    pub css_path: Option<String>,
}

impl Default for TurboConfig {
    fn default() -> Self {
        Self {
            name: "TurboApp".to_string(),
            streaming: true,
            default_title: "TurboCommerce".to_string(),
            css_path: None,
        }
    }
}

impl TurboConfig {
    /// Create a new configuration with the given app name.
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            ..Default::default()
        }
    }

    /// Set the default page title.
    pub fn with_title(mut self, title: impl Into<String>) -> Self {
        self.default_title = title.into();
        self
    }

    /// Set the CSS file path.
    pub fn with_css(mut self, path: impl Into<String>) -> Self {
        self.css_path = Some(path.into());
        self
    }

    /// Enable or disable streaming SSR.
    pub fn with_streaming(mut self, enabled: bool) -> Self {
        self.streaming = enabled;
        self
    }
}

/// TurboCommerce application builder.
///
/// Use this to configure and build your application.
///
/// # Example
///
/// ```rust,ignore
/// let app = TurboApp::new("my-store")
///     .with_title("My Store")
///     .with_css("/pkg/style.css")
///     .build();
/// ```
#[derive(Debug)]
pub struct TurboApp {
    config: TurboConfig,
    routes: RouteRegistry,
}

impl TurboApp {
    /// Create a new TurboApp with the given name.
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            config: TurboConfig::new(name),
            routes: RouteRegistry::new(),
        }
    }

    /// Set the default page title.
    pub fn with_title(mut self, title: impl Into<String>) -> Self {
        self.config = self.config.with_title(title);
        self
    }

    /// Set the CSS file path.
    pub fn with_css(mut self, path: impl Into<String>) -> Self {
        self.config = self.config.with_css(path);
        self
    }

    /// Enable or disable streaming SSR.
    pub fn with_streaming(mut self, enabled: bool) -> Self {
        self.config = self.config.with_streaming(enabled);
        self
    }

    /// Register a route manually.
    pub fn route(mut self, path: impl Into<String>, component: impl Into<String>) -> Self {
        self.routes.register(path, component);
        self
    }

    /// Get the configuration.
    pub fn config(&self) -> &TurboConfig {
        &self.config
    }

    /// Get the route registry.
    pub fn routes(&self) -> &RouteRegistry {
        &self.routes
    }

    /// Build the application configuration.
    pub fn build(self) -> (TurboConfig, RouteRegistry) {
        (self.config, self.routes)
    }
}
