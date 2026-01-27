//! Sandbox configuration for workload isolation.

use std::time::Duration;

use serde::{Deserialize, Serialize};

/// Sandbox configuration defining resource constraints for workload execution.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SandboxConfig {
    /// Maximum memory in bytes the workload can use.
    pub memory_limit_bytes: u64,
    /// Maximum execution time before timeout.
    pub execution_timeout: Duration,
    /// Maximum CPU time (if supported by runtime).
    pub cpu_time_limit: Option<Duration>,
    /// Maximum number of file descriptors.
    pub max_file_descriptors: u32,
    /// Whether to allow filesystem access.
    pub allow_filesystem: bool,
    /// Whether to allow network access.
    pub allow_network: bool,
    /// Whether to allow environment variable access.
    pub allow_env_vars: bool,
    /// Specific environment variables to expose (if allow_env_vars is false).
    pub exposed_env_vars: Vec<String>,
    /// Whether to enable WASI preview 2 features.
    pub wasi_preview2: bool,
}

impl Default for SandboxConfig {
    fn default() -> Self {
        Self {
            memory_limit_bytes: 128 * 1024 * 1024, // 128 MB
            execution_timeout: Duration::from_secs(30),
            cpu_time_limit: None,
            max_file_descriptors: 16,
            allow_filesystem: false,
            allow_network: true, // Required for HTTP handlers
            allow_env_vars: false,
            exposed_env_vars: Vec::new(),
            wasi_preview2: true,
        }
    }
}

impl SandboxConfig {
    /// Create a new sandbox config with defaults.
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a minimal sandbox with strict limits.
    pub fn minimal() -> Self {
        Self {
            memory_limit_bytes: 32 * 1024 * 1024, // 32 MB
            execution_timeout: Duration::from_secs(5),
            cpu_time_limit: Some(Duration::from_secs(2)),
            max_file_descriptors: 4,
            allow_filesystem: false,
            allow_network: true,
            allow_env_vars: false,
            exposed_env_vars: Vec::new(),
            wasi_preview2: true,
        }
    }

    /// Create a permissive sandbox for development.
    pub fn development() -> Self {
        Self {
            memory_limit_bytes: 512 * 1024 * 1024, // 512 MB
            execution_timeout: Duration::from_secs(120),
            cpu_time_limit: None,
            max_file_descriptors: 64,
            allow_filesystem: true,
            allow_network: true,
            allow_env_vars: true,
            exposed_env_vars: Vec::new(),
            wasi_preview2: true,
        }
    }

    /// Set memory limit in megabytes.
    pub fn with_memory_limit_mb(mut self, mb: u64) -> Self {
        self.memory_limit_bytes = mb * 1024 * 1024;
        self
    }

    /// Set memory limit in bytes.
    pub fn with_memory_limit_bytes(mut self, bytes: u64) -> Self {
        self.memory_limit_bytes = bytes;
        self
    }

    /// Set execution timeout in milliseconds.
    pub fn with_execution_timeout_ms(mut self, ms: u64) -> Self {
        self.execution_timeout = Duration::from_millis(ms);
        self
    }

    /// Set execution timeout.
    pub fn with_execution_timeout(mut self, timeout: Duration) -> Self {
        self.execution_timeout = timeout;
        self
    }

    /// Set CPU time limit.
    pub fn with_cpu_time_limit(mut self, limit: Duration) -> Self {
        self.cpu_time_limit = Some(limit);
        self
    }

    /// Set maximum file descriptors.
    pub fn with_max_file_descriptors(mut self, max: u32) -> Self {
        self.max_file_descriptors = max;
        self
    }

    /// Allow or deny filesystem access.
    pub fn allow_filesystem(mut self, allow: bool) -> Self {
        self.allow_filesystem = allow;
        self
    }

    /// Allow or deny network access.
    pub fn allow_network(mut self, allow: bool) -> Self {
        self.allow_network = allow;
        self
    }

    /// Allow or deny environment variable access.
    pub fn allow_env_vars(mut self, allow: bool) -> Self {
        self.allow_env_vars = allow;
        self
    }

    /// Expose specific environment variables.
    pub fn expose_env_var(mut self, name: impl Into<String>) -> Self {
        self.exposed_env_vars.push(name.into());
        self
    }

    /// Expose multiple environment variables.
    pub fn expose_env_vars(mut self, names: Vec<String>) -> Self {
        self.exposed_env_vars.extend(names);
        self
    }

    /// Validate the configuration.
    pub fn validate(&self) -> Result<(), SandboxConfigError> {
        if self.memory_limit_bytes == 0 {
            return Err(SandboxConfigError::InvalidMemoryLimit);
        }

        if self.memory_limit_bytes > 4 * 1024 * 1024 * 1024 {
            // 4 GB max
            return Err(SandboxConfigError::MemoryLimitTooHigh);
        }

        if self.execution_timeout.is_zero() {
            return Err(SandboxConfigError::InvalidTimeout);
        }

        if self.execution_timeout > Duration::from_secs(300) {
            // 5 min max
            return Err(SandboxConfigError::TimeoutTooLong);
        }

        Ok(())
    }

    /// Check if an environment variable should be exposed.
    pub fn should_expose_env(&self, name: &str) -> bool {
        if self.allow_env_vars {
            return true;
        }
        self.exposed_env_vars.iter().any(|v| v == name)
    }

    /// Get memory limit as human-readable string.
    pub fn memory_limit_display(&self) -> String {
        let bytes = self.memory_limit_bytes;
        if bytes >= 1024 * 1024 * 1024 {
            format!("{} GB", bytes / (1024 * 1024 * 1024))
        } else if bytes >= 1024 * 1024 {
            format!("{} MB", bytes / (1024 * 1024))
        } else if bytes >= 1024 {
            format!("{} KB", bytes / 1024)
        } else {
            format!("{} bytes", bytes)
        }
    }
}

/// Errors in sandbox configuration.
#[derive(Debug, Clone, thiserror::Error)]
pub enum SandboxConfigError {
    #[error("memory limit must be greater than 0")]
    InvalidMemoryLimit,

    #[error("memory limit exceeds maximum allowed (4 GB)")]
    MemoryLimitTooHigh,

    #[error("execution timeout must be greater than 0")]
    InvalidTimeout,

    #[error("execution timeout exceeds maximum allowed (5 minutes)")]
    TimeoutTooLong,
}

/// Runtime sandbox state tracking.
#[derive(Debug, Clone, Default)]
pub struct SandboxState {
    /// Current memory usage in bytes.
    pub memory_used_bytes: u64,
    /// Elapsed execution time.
    pub execution_time: Duration,
    /// Number of open file descriptors.
    pub open_file_descriptors: u32,
    /// Number of network connections made.
    pub network_connections: u32,
}

impl SandboxState {
    /// Create a new sandbox state.
    pub fn new() -> Self {
        Self::default()
    }

    /// Check if memory limit is exceeded.
    pub fn is_memory_exceeded(&self, config: &SandboxConfig) -> bool {
        self.memory_used_bytes > config.memory_limit_bytes
    }

    /// Check if execution timeout is exceeded.
    pub fn is_timeout_exceeded(&self, config: &SandboxConfig) -> bool {
        self.execution_time > config.execution_timeout
    }

    /// Check if file descriptor limit is exceeded.
    pub fn is_fd_limit_exceeded(&self, config: &SandboxConfig) -> bool {
        self.open_file_descriptors > config.max_file_descriptors
    }

    /// Get memory usage as percentage.
    pub fn memory_usage_percent(&self, config: &SandboxConfig) -> f64 {
        (self.memory_used_bytes as f64 / config.memory_limit_bytes as f64) * 100.0
    }

    /// Get execution time as percentage of timeout.
    pub fn execution_time_percent(&self, config: &SandboxConfig) -> f64 {
        (self.execution_time.as_secs_f64() / config.execution_timeout.as_secs_f64()) * 100.0
    }
}

/// Violation type when sandbox limits are exceeded.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "details")]
pub enum SandboxViolation {
    /// Memory limit exceeded.
    MemoryExceeded { used: u64, limit: u64 },
    /// Execution timeout exceeded.
    TimeoutExceeded { elapsed_ms: u64, limit_ms: u64 },
    /// CPU time limit exceeded.
    CpuTimeExceeded { used_ms: u64, limit_ms: u64 },
    /// File descriptor limit exceeded.
    FileDescriptorExceeded { count: u32, limit: u32 },
    /// Unauthorized filesystem access.
    FilesystemAccessDenied { path: String },
    /// Unauthorized network access.
    NetworkAccessDenied { host: String },
    /// Unauthorized environment variable access.
    EnvVarAccessDenied { name: String },
}

impl std::fmt::Display for SandboxViolation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MemoryExceeded { used, limit } => {
                write!(f, "memory exceeded: {} / {} bytes", used, limit)
            }
            Self::TimeoutExceeded { elapsed_ms, limit_ms } => {
                write!(f, "timeout exceeded: {} / {} ms", elapsed_ms, limit_ms)
            }
            Self::CpuTimeExceeded { used_ms, limit_ms } => {
                write!(f, "CPU time exceeded: {} / {} ms", used_ms, limit_ms)
            }
            Self::FileDescriptorExceeded { count, limit } => {
                write!(f, "file descriptor limit exceeded: {} / {}", count, limit)
            }
            Self::FilesystemAccessDenied { path } => {
                write!(f, "filesystem access denied: {}", path)
            }
            Self::NetworkAccessDenied { host } => {
                write!(f, "network access denied: {}", host)
            }
            Self::EnvVarAccessDenied { name } => {
                write!(f, "environment variable access denied: {}", name)
            }
        }
    }
}
