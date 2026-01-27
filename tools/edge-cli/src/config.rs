//! CLI configuration.

use std::collections::HashMap;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

/// CLI configuration file.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CliConfig {
    /// Workload configuration.
    #[serde(default)]
    pub workload: WorkloadConfig,

    /// Build configuration.
    #[serde(default)]
    pub build: BuildConfig,

    /// Deployment configuration.
    #[serde(default)]
    pub deploy: DeployConfig,

    /// Environment-specific overrides.
    #[serde(default)]
    pub environments: HashMap<String, EnvironmentConfig>,
}

impl CliConfig {
    /// Load config from a file.
    pub fn load(path: &str) -> Result<Self> {
        let content = std::fs::read_to_string(path)
            .with_context(|| format!("Failed to read config file: {}", path))?;

        if path.ends_with(".json") {
            serde_json::from_str(&content)
                .with_context(|| format!("Failed to parse JSON config: {}", path))
        } else {
            toml::from_str(&content)
                .with_context(|| format!("Failed to parse TOML config: {}", path))
        }
    }

    /// Save config to a file.
    pub fn save(&self, path: &str) -> Result<()> {
        let content = if path.ends_with(".json") {
            serde_json::to_string_pretty(self)?
        } else {
            toml::to_string_pretty(self)?
        };

        std::fs::write(path, content)
            .with_context(|| format!("Failed to write config file: {}", path))
    }

    /// Get environment-specific config.
    pub fn for_environment(&self, env: &str) -> CliConfig {
        let mut config = self.clone();

        if let Some(env_config) = self.environments.get(env) {
            // Merge environment config
            if let Some(ref deploy) = env_config.deploy {
                config.deploy = deploy.clone();
            }
        }

        config
    }

    /// Create a default config for a new workload.
    pub fn default_for_workload(name: &str) -> Self {
        Self {
            workload: WorkloadConfig {
                name: name.to_string(),
                version: "0.1.0".to_string(),
                description: None,
                authors: Vec::new(),
            },
            build: BuildConfig::default(),
            deploy: DeployConfig::default(),
            environments: HashMap::new(),
        }
    }
}

/// Workload metadata.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct WorkloadConfig {
    /// Workload name.
    pub name: String,

    /// Workload version.
    #[serde(default = "default_version")]
    pub version: String,

    /// Workload description.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// Workload authors.
    #[serde(default)]
    pub authors: Vec<String>,
}

fn default_version() -> String {
    "0.1.0".to_string()
}

/// Build configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildConfig {
    /// Rust target (default: wasm32-wasip1).
    #[serde(default = "default_target")]
    pub target: String,

    /// Build profile (default: release).
    #[serde(default = "default_profile")]
    pub profile: String,

    /// Additional cargo features to enable.
    #[serde(default)]
    pub features: Vec<String>,

    /// Whether to run wasm-opt after build.
    #[serde(default = "default_true")]
    pub optimize: bool,

    /// Extra cargo arguments.
    #[serde(default)]
    pub cargo_args: Vec<String>,
}

fn default_target() -> String {
    "wasm32-wasip1".to_string()
}

fn default_profile() -> String {
    "release".to_string()
}

fn default_true() -> bool {
    true
}

impl Default for BuildConfig {
    fn default() -> Self {
        Self {
            target: default_target(),
            profile: default_profile(),
            features: Vec::new(),
            optimize: true,
            cargo_args: Vec::new(),
        }
    }
}

/// Deployment configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeployConfig {
    /// Spin platform URL.
    #[serde(default)]
    pub platform_url: Option<String>,

    /// Application name on the platform.
    #[serde(default)]
    pub app_name: Option<String>,

    /// Environment variables to set.
    #[serde(default)]
    pub env_vars: HashMap<String, String>,

    /// Routes to register.
    #[serde(default)]
    pub routes: Vec<RouteConfig>,

    /// Whether to use canary deployments.
    #[serde(default)]
    pub canary: bool,

    /// Canary percentage (0-100).
    #[serde(default)]
    pub canary_percentage: u8,

    /// Number of versions to keep.
    #[serde(default = "default_versions_to_keep")]
    pub versions_to_keep: usize,
}

fn default_versions_to_keep() -> usize {
    5
}

impl Default for DeployConfig {
    fn default() -> Self {
        Self {
            platform_url: None,
            app_name: None,
            env_vars: HashMap::new(),
            routes: Vec::new(),
            canary: false,
            canary_percentage: 10,
            versions_to_keep: default_versions_to_keep(),
        }
    }
}

/// Route configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RouteConfig {
    /// Route path pattern.
    pub path: String,

    /// HTTP methods (default: GET).
    #[serde(default = "default_methods")]
    pub methods: Vec<String>,

    /// Handler component.
    #[serde(default)]
    pub handler: Option<String>,
}

fn default_methods() -> Vec<String> {
    vec!["GET".to_string()]
}

/// Environment-specific configuration overrides.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct EnvironmentConfig {
    /// Deploy config override.
    #[serde(default)]
    pub deploy: Option<DeployConfig>,
}

/// Generate a default edge.toml config file.
pub fn generate_default_config(name: &str) -> String {
    format!(
        r#"# Edge workload configuration

[workload]
name = "{name}"
version = "0.1.0"
description = "An edge streaming SSR workload"

[build]
target = "wasm32-wasip1"
profile = "release"
optimize = true

[deploy]
# platform_url = "https://cloud.fermyon.com"
# app_name = "{name}"
canary = false
versions_to_keep = 5

[[deploy.routes]]
path = "/hello"
methods = ["GET"]

[environments.staging]
[environments.staging.deploy]
# app_name = "{name}-staging"
canary = true
canary_percentage = 20

[environments.production]
[environments.production.deploy]
# app_name = "{name}"
canary = false
"#,
        name = name
    )
}
