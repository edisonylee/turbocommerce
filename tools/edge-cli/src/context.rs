//! CLI execution context.

use std::path::PathBuf;

use anyhow::{Context as _, Result};

use crate::config::CliConfig;
use crate::output::Output;

/// Execution context for CLI commands.
pub struct Context {
    /// CLI configuration.
    pub config: CliConfig,
    /// Output handler.
    pub output: Output,
    /// Working directory.
    pub cwd: PathBuf,
}

impl Context {
    /// Load context from config file.
    pub fn load(config_path: Option<&str>, output: Output) -> Result<Self> {
        let cwd = std::env::current_dir().context("Failed to get current directory")?;

        let config = if let Some(path) = config_path {
            CliConfig::load(path)?
        } else {
            // Try to find config in current directory or parent directories
            Self::find_config(&cwd).unwrap_or_default()
        };

        Ok(Self { config, output, cwd })
    }

    /// Find config file in directory tree.
    fn find_config(start: &PathBuf) -> Option<CliConfig> {
        let config_names = ["edge.toml", ".edge.toml", "edge.json"];

        let mut current = start.clone();
        loop {
            for name in &config_names {
                let config_path = current.join(name);
                if config_path.exists() {
                    if let Ok(config) = CliConfig::load(config_path.to_str()?) {
                        return Some(config);
                    }
                }
            }

            if !current.pop() {
                break;
            }
        }

        None
    }

    /// Get the workload directory.
    pub fn workload_dir(&self) -> PathBuf {
        self.cwd.clone()
    }

    /// Get the build output directory.
    pub fn build_dir(&self) -> PathBuf {
        self.cwd.join("target").join("wasm32-wasip1").join("release")
    }

    /// Get the cache directory.
    pub fn cache_dir(&self) -> Result<PathBuf> {
        let cache = dirs_path().join("edge-cli").join("cache");
        std::fs::create_dir_all(&cache)?;
        Ok(cache)
    }

    /// Get the recordings directory.
    pub fn recordings_dir(&self) -> Result<PathBuf> {
        let recordings = self.cwd.join(".edge").join("recordings");
        std::fs::create_dir_all(&recordings)?;
        Ok(recordings)
    }

    /// Resolve a path relative to the working directory.
    pub fn resolve_path(&self, path: &str) -> PathBuf {
        if PathBuf::from(path).is_absolute() {
            PathBuf::from(path)
        } else {
            self.cwd.join(path)
        }
    }
}

/// Get the platform-specific data directory.
fn dirs_path() -> PathBuf {
    if let Some(home) = std::env::var_os("HOME") {
        PathBuf::from(home).join(".local").join("share")
    } else {
        PathBuf::from("/tmp")
    }
}
