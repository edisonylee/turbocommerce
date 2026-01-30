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

#[cfg(test)]
mod tests {
    use super::*;

    // === TurboConfig Tests ===

    #[test]
    fn test_turbo_config_default() {
        let config = TurboConfig::default();

        assert_eq!(config.name, "TurboApp");
        assert!(config.streaming);
        assert_eq!(config.default_title, "TurboCommerce");
        assert!(config.css_path.is_none());
    }

    #[test]
    fn test_turbo_config_new() {
        let config = TurboConfig::new("MyStore");

        assert_eq!(config.name, "MyStore");
        assert!(config.streaming); // Default
    }

    #[test]
    fn test_turbo_config_with_title() {
        let config = TurboConfig::new("App").with_title("Custom Title");

        assert_eq!(config.default_title, "Custom Title");
    }

    #[test]
    fn test_turbo_config_with_css() {
        let config = TurboConfig::new("App").with_css("/styles/main.css");

        assert_eq!(config.css_path, Some("/styles/main.css".to_string()));
    }

    #[test]
    fn test_turbo_config_with_streaming() {
        let config = TurboConfig::new("App").with_streaming(false);

        assert!(!config.streaming);
    }

    #[test]
    fn test_turbo_config_builder_chain() {
        let config = TurboConfig::new("Shop")
            .with_title("My Shop")
            .with_css("/pkg/style.css")
            .with_streaming(true);

        assert_eq!(config.name, "Shop");
        assert_eq!(config.default_title, "My Shop");
        assert_eq!(config.css_path, Some("/pkg/style.css".to_string()));
        assert!(config.streaming);
    }

    #[test]
    fn test_turbo_config_clone() {
        let config = TurboConfig::new("App")
            .with_title("Title")
            .with_css("/style.css");

        let cloned = config.clone();
        assert_eq!(cloned.name, config.name);
        assert_eq!(cloned.default_title, config.default_title);
    }

    // === TurboApp Tests ===

    #[test]
    fn test_turbo_app_new() {
        let app = TurboApp::new("TestApp");

        assert_eq!(app.config().name, "TestApp");
        assert!(app.routes().routes().is_empty());
    }

    #[test]
    fn test_turbo_app_with_title() {
        let app = TurboApp::new("App").with_title("Custom Title");

        assert_eq!(app.config().default_title, "Custom Title");
    }

    #[test]
    fn test_turbo_app_with_css() {
        let app = TurboApp::new("App").with_css("/pkg/style.css");

        assert_eq!(app.config().css_path, Some("/pkg/style.css".to_string()));
    }

    #[test]
    fn test_turbo_app_with_streaming() {
        let app = TurboApp::new("App").with_streaming(false);

        assert!(!app.config().streaming);
    }

    #[test]
    fn test_turbo_app_route() {
        let app = TurboApp::new("App")
            .route("/", "HomePage")
            .route("/about", "AboutPage");

        let routes = app.routes().routes();
        assert_eq!(routes.len(), 2);
    }

    #[test]
    fn test_turbo_app_build() {
        let app = TurboApp::new("BuildTest")
            .with_title("Test")
            .route("/", "Home");

        let (config, routes) = app.build();

        assert_eq!(config.name, "BuildTest");
        assert_eq!(config.default_title, "Test");
        assert_eq!(routes.routes().len(), 1);
    }

    #[test]
    fn test_turbo_app_full_builder() {
        let (config, routes) = TurboApp::new("E-Commerce")
            .with_title("My Store")
            .with_css("/assets/style.css")
            .with_streaming(true)
            .route("/", "Home")
            .route("/products", "Products")
            .route("/cart", "Cart")
            .build();

        assert_eq!(config.name, "E-Commerce");
        assert_eq!(config.default_title, "My Store");
        assert_eq!(config.css_path, Some("/assets/style.css".to_string()));
        assert!(config.streaming);
        assert_eq!(routes.routes().len(), 3);
    }
}
