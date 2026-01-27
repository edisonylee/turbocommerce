//! Version management commands.

use std::fs;

use anyhow::{bail, Result};
use chrono::{DateTime, Utc};

use super::{VersionsArgs, VersionsCommand};
use crate::context::Context;
use crate::output::status_badge;

/// Run the versions command.
pub async fn run(args: VersionsArgs, ctx: &Context) -> Result<()> {
    match args.command {
        Some(VersionsCommand::List) | None => list_versions(&args, ctx).await,
        Some(VersionsCommand::Show { version }) => show_version(&version, &args.env, ctx).await,
        Some(VersionsCommand::Promote { version }) => promote_version(&version, &args.env, ctx).await,
        Some(VersionsCommand::Delete { version, yes }) => {
            delete_version(&version, &args.env, yes, ctx).await
        }
    }
}

async fn list_versions(args: &VersionsArgs, ctx: &Context) -> Result<()> {
    ctx.output.header(&format!("Versions for {}", args.env));

    let deployments_dir = ctx.workload_dir().join(".edge").join("deployments");

    if !deployments_dir.exists() {
        ctx.output.info("No deployments found.");
        ctx.output.info("Run `edge deploy` to create your first deployment.");
        return Ok(());
    }

    let mut versions: Vec<VersionInfo> = Vec::new();

    for entry in fs::read_dir(&deployments_dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.extension().map_or(false, |e| e == "json") {
            let filename = path.file_stem().unwrap().to_string_lossy();

            // Filter by environment
            if !filename.starts_with(&format!("{}-", args.env)) {
                continue;
            }

            if let Ok(content) = fs::read_to_string(&path) {
                if let Ok(record) = serde_json::from_str::<DeploymentRecord>(&content) {
                    let status = determine_status(&record);
                    versions.push(VersionInfo {
                        version: record.version,
                        timestamp: record.timestamp,
                        url: record.url,
                        canary: record.canary,
                        status,
                    });
                }
            }
        }
    }

    // Sort by timestamp (newest first)
    versions.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));

    // Apply limit
    if let Some(limit) = args.limit {
        versions.truncate(limit);
    }

    if versions.is_empty() {
        ctx.output.info(&format!("No versions found for environment '{}'", args.env));
        return Ok(());
    }

    if ctx.output.is_json() {
        ctx.output.json(&versions);
        return Ok(());
    }

    // Print table header
    ctx.output.table_row(
        &["VERSION", "TIMESTAMP", "STATUS", "URL"],
        &[20, 25, 12, 40],
    );
    ctx.output.info(&"-".repeat(100));

    for v in &versions {
        let timestamp = format_timestamp(&v.timestamp);
        let status = status_badge(&v.status);
        let url = v.url.as_deref().unwrap_or("-");

        ctx.output.table_row(
            &[&v.version, &timestamp, &status, url],
            &[20, 25, 12, 40],
        );
    }

    ctx.output.info("");
    ctx.output.info(&format!("Total: {} version(s)", versions.len()));

    Ok(())
}

async fn show_version(version: &str, env: &str, ctx: &Context) -> Result<()> {
    let deployments_dir = ctx.workload_dir().join(".edge").join("deployments");
    let path = deployments_dir.join(format!("{}-{}.json", env, version));

    if !path.exists() {
        bail!("Version '{}' not found in environment '{}'", version, env);
    }

    let content = fs::read_to_string(&path)?;
    let record: DeploymentRecord = serde_json::from_str(&content)?;

    if ctx.output.is_json() {
        ctx.output.json(&record);
        return Ok(());
    }

    ctx.output.header(&format!("Version: {}", version));
    ctx.output.kv("Environment", &record.environment);
    ctx.output.kv("Application", &record.app_name);
    ctx.output.kv("Deployed", &format_timestamp(&record.timestamp));
    ctx.output.kv("Status", &status_badge(&determine_status(&record)));

    if let Some(url) = &record.url {
        ctx.output.kv("URL", url);
    }

    if record.canary {
        ctx.output.kv(
            "Canary",
            &format!("{}%", record.canary_percentage.unwrap_or(0)),
        );
    }

    Ok(())
}

async fn promote_version(version: &str, env: &str, ctx: &Context) -> Result<()> {
    ctx.output.header(&format!("Promoting {} to full rollout", version));

    let deployments_dir = ctx.workload_dir().join(".edge").join("deployments");
    let path = deployments_dir.join(format!("{}-{}.json", env, version));

    if !path.exists() {
        bail!("Version '{}' not found in environment '{}'", version, env);
    }

    let content = fs::read_to_string(&path)?;
    let mut record: DeploymentRecord = serde_json::from_str(&content)?;

    if !record.canary {
        ctx.output.info("Version is already at 100% rollout");
        return Ok(());
    }

    // Update record
    record.canary = false;
    record.canary_percentage = None;

    let json = serde_json::to_string_pretty(&record)?;
    fs::write(&path, json)?;

    ctx.output.success(&format!(
        "Version '{}' promoted to 100% in '{}'",
        version, env
    ));

    // In a real implementation, this would also call the platform API
    ctx.output.warn("Note: This updates the local record only. Use `spin deploy` to update the actual deployment.");

    Ok(())
}

async fn delete_version(version: &str, env: &str, yes: bool, ctx: &Context) -> Result<()> {
    let deployments_dir = ctx.workload_dir().join(".edge").join("deployments");
    let path = deployments_dir.join(format!("{}-{}.json", env, version));

    if !path.exists() {
        bail!("Version '{}' not found in environment '{}'", version, env);
    }

    if !yes {
        use dialoguer::Confirm;
        let confirmed = Confirm::new()
            .with_prompt(format!("Delete version '{}'?", version))
            .default(false)
            .interact()?;

        if !confirmed {
            ctx.output.warn("Cancelled");
            return Ok(());
        }
    }

    fs::remove_file(&path)?;
    ctx.output.success(&format!("Deleted version '{}' from '{}'", version, env));

    Ok(())
}

#[derive(serde::Serialize, serde::Deserialize)]
struct DeploymentRecord {
    version: String,
    environment: String,
    timestamp: String,
    app_name: String,
    url: Option<String>,
    canary: bool,
    canary_percentage: Option<u8>,
}

#[derive(serde::Serialize)]
struct VersionInfo {
    version: String,
    timestamp: String,
    url: Option<String>,
    canary: bool,
    status: String,
}

fn determine_status(record: &DeploymentRecord) -> String {
    if record.canary {
        format!("canary-{}%", record.canary_percentage.unwrap_or(0))
    } else {
        "active".to_string()
    }
}

fn format_timestamp(ts: &str) -> String {
    if let Ok(dt) = DateTime::parse_from_rfc3339(ts) {
        let utc: DateTime<Utc> = dt.into();
        utc.format("%Y-%m-%d %H:%M:%S UTC").to_string()
    } else {
        ts.to_string()
    }
}
