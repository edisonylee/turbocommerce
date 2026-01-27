//! Rollback to a previous version.

use std::fs;

use anyhow::{bail, Result};
use dialoguer::{Confirm, Select};

use super::RollbackArgs;
use crate::context::Context;

/// Run the rollback command.
pub async fn run(args: RollbackArgs, ctx: &Context) -> Result<()> {
    ctx.output.header(&format!("Rollback in {}", args.env));

    let deployments_dir = ctx.workload_dir().join(".edge").join("deployments");

    if !deployments_dir.exists() {
        bail!("No deployments found. Nothing to rollback to.");
    }

    // Get list of versions
    let mut versions: Vec<(String, String)> = Vec::new();

    for entry in fs::read_dir(&deployments_dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.extension().map_or(false, |e| e == "json") {
            let filename = path.file_stem().unwrap().to_string_lossy();

            if !filename.starts_with(&format!("{}-", args.env)) {
                continue;
            }

            if let Ok(content) = fs::read_to_string(&path) {
                if let Ok(record) = serde_json::from_str::<DeploymentRecord>(&content) {
                    versions.push((record.version, record.timestamp));
                }
            }
        }
    }

    // Sort by timestamp (newest first)
    versions.sort_by(|a, b| b.1.cmp(&a.1));

    if versions.len() < 2 {
        bail!("Not enough versions to rollback. Need at least 2 versions.");
    }

    // Determine target version
    let target_version = if let Some(ref to) = args.to {
        // Validate the specified version exists
        if !versions.iter().any(|(v, _)| v == to) {
            bail!("Version '{}' not found", to);
        }
        to.clone()
    } else {
        // Interactive selection or use previous version
        if args.yes {
            // Use the second most recent version (rollback to previous)
            versions[1].0.clone()
        } else {
            // Interactive selection
            let items: Vec<String> = versions
                .iter()
                .map(|(v, ts)| format!("{} ({})", v, ts))
                .collect();

            let selection = Select::new()
                .with_prompt("Select version to rollback to")
                .items(&items)
                .default(1) // Default to previous version
                .interact()?;

            versions[selection].0.clone()
        }
    };

    ctx.output.info(&format!("Rolling back to version: {}", target_version));

    // Confirmation
    if !args.yes && !args.dry_run {
        let current = &versions[0].0;
        ctx.output.warn(&format!(
            "This will replace {} with {}",
            current, target_version
        ));

        let confirmed = Confirm::new()
            .with_prompt("Proceed with rollback?")
            .default(false)
            .interact()?;

        if !confirmed {
            ctx.output.warn("Rollback cancelled");
            return Ok(());
        }
    }

    if args.dry_run {
        ctx.output.info("Dry run - no changes made");
        ctx.output.success(&format!(
            "Would rollback to version: {}",
            target_version
        ));
        return Ok(());
    }

    // Perform rollback
    ctx.output.step(1, 3, "Loading target version configuration");

    let target_path = deployments_dir.join(format!("{}-{}.json", args.env, target_version));
    let content = fs::read_to_string(&target_path)?;
    let record: DeploymentRecord = serde_json::from_str(&content)?;

    ctx.output.step(2, 3, "Deploying previous version");

    // In a real implementation, this would:
    // 1. Load the WASM artifact for the target version (from storage/registry)
    // 2. Call spin deploy with the old artifact
    // For now, we just inform the user

    ctx.output.warn("Note: Automatic rollback requires artifact storage.");
    ctx.output.info("To complete the rollback manually:");
    ctx.output.list_item(&format!("git checkout {}", target_version));
    ctx.output.list_item("edge build");
    ctx.output.list_item(&format!("edge deploy --env {} --tag {}-rollback", args.env, target_version));

    ctx.output.step(3, 3, "Recording rollback");

    // Create a rollback record
    let rollback_record = RollbackRecord {
        from_version: versions[0].0.clone(),
        to_version: target_version.clone(),
        environment: args.env.clone(),
        timestamp: chrono::Utc::now().to_rfc3339(),
        reason: None,
    };

    let rollbacks_dir = ctx.workload_dir().join(".edge").join("rollbacks");
    fs::create_dir_all(&rollbacks_dir)?;

    let rollback_path = rollbacks_dir.join(format!(
        "{}-{}.json",
        args.env,
        chrono::Utc::now().format("%Y%m%d%H%M%S")
    ));
    let json = serde_json::to_string_pretty(&rollback_record)?;
    fs::write(&rollback_path, json)?;

    ctx.output.success(&format!(
        "Rollback to {} initiated",
        target_version
    ));

    if let Some(url) = record.url {
        ctx.output.kv("URL", &url);
    }

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

#[derive(serde::Serialize, serde::Deserialize)]
struct RollbackRecord {
    from_version: String,
    to_version: String,
    environment: String,
    timestamp: String,
    reason: Option<String>,
}
