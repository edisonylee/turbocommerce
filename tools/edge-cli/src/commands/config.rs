//! Configuration management commands.

use std::fs;

use anyhow::{bail, Result};

use super::{ConfigArgs, ConfigCommand};
use crate::config::{generate_default_config, CliConfig};
use crate::context::Context;

/// Run the config command.
pub async fn run(args: ConfigArgs, ctx: &Context) -> Result<()> {
    match args.command {
        ConfigCommand::Show => show_config(ctx).await,
        ConfigCommand::Get { key } => get_config(&key, ctx).await,
        ConfigCommand::Set { key, value } => set_config(&key, &value, ctx).await,
        ConfigCommand::Init { force } => init_config(force, ctx).await,
        ConfigCommand::Validate => validate_config(ctx).await,
    }
}

async fn show_config(ctx: &Context) -> Result<()> {
    ctx.output.header("Current Configuration");

    if ctx.output.is_json() {
        ctx.output.json(&ctx.config);
        return Ok(());
    }

    // Workload section
    ctx.output.info("");
    ctx.output.info("[workload]");
    ctx.output.kv("name", &ctx.config.workload.name);
    ctx.output.kv("version", &ctx.config.workload.version);
    if let Some(ref desc) = ctx.config.workload.description {
        ctx.output.kv("description", desc);
    }

    // Build section
    ctx.output.info("");
    ctx.output.info("[build]");
    ctx.output.kv("target", &ctx.config.build.target);
    ctx.output.kv("profile", &ctx.config.build.profile);
    ctx.output.kv("optimize", &ctx.config.build.optimize.to_string());
    if !ctx.config.build.features.is_empty() {
        ctx.output.kv("features", &ctx.config.build.features.join(", "));
    }

    // Deploy section
    ctx.output.info("");
    ctx.output.info("[deploy]");
    if let Some(ref url) = ctx.config.deploy.platform_url {
        ctx.output.kv("platform_url", url);
    }
    if let Some(ref name) = ctx.config.deploy.app_name {
        ctx.output.kv("app_name", name);
    }
    ctx.output.kv("canary", &ctx.config.deploy.canary.to_string());
    ctx.output.kv(
        "versions_to_keep",
        &ctx.config.deploy.versions_to_keep.to_string(),
    );

    // Environment variables
    if !ctx.config.deploy.env_vars.is_empty() {
        ctx.output.info("");
        ctx.output.info("[deploy.env_vars]");
        for (key, value) in &ctx.config.deploy.env_vars {
            ctx.output.kv(key, value);
        }
    }

    // Routes
    if !ctx.config.deploy.routes.is_empty() {
        ctx.output.info("");
        ctx.output.info("[[deploy.routes]]");
        for route in &ctx.config.deploy.routes {
            ctx.output.kv("path", &route.path);
            ctx.output.kv("methods", &route.methods.join(", "));
        }
    }

    // Environments
    if !ctx.config.environments.is_empty() {
        ctx.output.info("");
        ctx.output.info("Environments:");
        for env in ctx.config.environments.keys() {
            ctx.output.list_item(env);
        }
    }

    Ok(())
}

async fn get_config(key: &str, ctx: &Context) -> Result<()> {
    let value = get_config_value(&ctx.config, key)?;

    if ctx.output.is_json() {
        println!(r#"{{"key": "{}", "value": {}}}"#, key, value);
    } else {
        println!("{}", value);
    }

    Ok(())
}

async fn set_config(key: &str, value: &str, ctx: &Context) -> Result<()> {
    // Find config file
    let config_path = find_config_file(&ctx.cwd)?;

    // Load current config
    let content = fs::read_to_string(&config_path)?;
    let mut config: CliConfig = if config_path.ends_with(".json") {
        serde_json::from_str(&content)?
    } else {
        toml::from_str(&content)?
    };

    // Set the value
    set_config_value(&mut config, key, value)?;

    // Save config
    let new_content = if config_path.ends_with(".json") {
        serde_json::to_string_pretty(&config)?
    } else {
        toml::to_string_pretty(&config)?
    };

    fs::write(&config_path, new_content)?;

    ctx.output.success(&format!("Set {} = {}", key, value));

    Ok(())
}

async fn init_config(force: bool, ctx: &Context) -> Result<()> {
    let config_path = ctx.cwd.join("edge.toml");

    if config_path.exists() && !force {
        bail!(
            "Config file already exists: {}. Use --force to overwrite.",
            config_path.display()
        );
    }

    // Determine workload name from directory or Cargo.toml
    let name = if let Ok(cargo_content) = fs::read_to_string(ctx.cwd.join("Cargo.toml")) {
        // Try to extract name from Cargo.toml
        extract_cargo_name(&cargo_content).unwrap_or_else(|| {
            ctx.cwd
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("my-workload")
                .to_string()
        })
    } else {
        ctx.cwd
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("my-workload")
            .to_string()
    };

    let content = generate_default_config(&name);
    fs::write(&config_path, content)?;

    ctx.output.success(&format!("Created: {}", config_path.display()));

    Ok(())
}

async fn validate_config(ctx: &Context) -> Result<()> {
    ctx.output.header("Validating configuration");

    let mut errors: Vec<String> = Vec::new();
    let mut warnings: Vec<String> = Vec::new();

    // Check workload name
    if ctx.config.workload.name.is_empty() {
        errors.push("workload.name is required".to_string());
    }

    // Check version format
    if !ctx.config.workload.version.contains('.') {
        warnings.push("workload.version should follow semver (e.g., 1.0.0)".to_string());
    }

    // Check build target
    if ctx.config.build.target != "wasm32-wasip1" && ctx.config.build.target != "wasm32-wasi" {
        warnings.push(format!(
            "build.target '{}' is unusual for Spin workloads",
            ctx.config.build.target
        ));
    }

    // Check deploy configuration
    if ctx.config.deploy.canary && ctx.config.deploy.canary_percentage > 100 {
        errors.push("deploy.canary_percentage must be 0-100".to_string());
    }

    // Check routes
    for (i, route) in ctx.config.deploy.routes.iter().enumerate() {
        if !route.path.starts_with('/') {
            errors.push(format!("deploy.routes[{}].path must start with '/'", i));
        }
    }

    // Print results
    if errors.is_empty() && warnings.is_empty() {
        ctx.output.success("Configuration is valid");
        return Ok(());
    }

    for error in &errors {
        ctx.output.error(&format!("Error: {}", error));
    }

    for warning in &warnings {
        ctx.output.warn(&format!("Warning: {}", warning));
    }

    if !errors.is_empty() {
        bail!("Configuration has {} error(s)", errors.len());
    }

    ctx.output.success("Configuration is valid (with warnings)");

    Ok(())
}

fn get_config_value(config: &CliConfig, key: &str) -> Result<String> {
    let parts: Vec<&str> = key.split('.').collect();

    match parts.as_slice() {
        ["workload", "name"] => Ok(format!("\"{}\"", config.workload.name)),
        ["workload", "version"] => Ok(format!("\"{}\"", config.workload.version)),
        ["workload", "description"] => Ok(config
            .workload
            .description
            .as_ref()
            .map(|d| format!("\"{}\"", d))
            .unwrap_or_else(|| "null".to_string())),
        ["build", "target"] => Ok(format!("\"{}\"", config.build.target)),
        ["build", "profile"] => Ok(format!("\"{}\"", config.build.profile)),
        ["build", "optimize"] => Ok(config.build.optimize.to_string()),
        ["deploy", "canary"] => Ok(config.deploy.canary.to_string()),
        ["deploy", "canary_percentage"] => Ok(config.deploy.canary_percentage.to_string()),
        ["deploy", "versions_to_keep"] => Ok(config.deploy.versions_to_keep.to_string()),
        _ => bail!("Unknown config key: {}", key),
    }
}

fn set_config_value(config: &mut CliConfig, key: &str, value: &str) -> Result<()> {
    let parts: Vec<&str> = key.split('.').collect();

    match parts.as_slice() {
        ["workload", "name"] => config.workload.name = value.to_string(),
        ["workload", "version"] => config.workload.version = value.to_string(),
        ["workload", "description"] => config.workload.description = Some(value.to_string()),
        ["build", "target"] => config.build.target = value.to_string(),
        ["build", "profile"] => config.build.profile = value.to_string(),
        ["build", "optimize"] => config.build.optimize = value.parse()?,
        ["deploy", "canary"] => config.deploy.canary = value.parse()?,
        ["deploy", "canary_percentage"] => config.deploy.canary_percentage = value.parse()?,
        ["deploy", "versions_to_keep"] => config.deploy.versions_to_keep = value.parse()?,
        _ => bail!("Unknown or read-only config key: {}", key),
    }

    Ok(())
}

fn find_config_file(cwd: &std::path::Path) -> Result<String> {
    for name in &["edge.toml", ".edge.toml", "edge.json"] {
        let path = cwd.join(name);
        if path.exists() {
            return Ok(path.to_string_lossy().to_string());
        }
    }
    bail!("No config file found. Run `edge config init` to create one.")
}

fn extract_cargo_name(content: &str) -> Option<String> {
    for line in content.lines() {
        let line = line.trim();
        if line.starts_with("name") && line.contains('=') {
            let value = line.split('=').nth(1)?.trim();
            let name = value.trim_matches('"').trim_matches('\'');
            return Some(name.to_string());
        }
    }
    None
}
