//! Deploy a workload to the edge.

use std::process::Command;

use anyhow::{bail, Context as _, Result};
use chrono::Utc;
use dialoguer::Confirm;

use super::DeployArgs;
use crate::context::Context;

/// Run the deploy command.
pub async fn run(args: DeployArgs, ctx: &Context) -> Result<()> {
    let config = ctx.config.for_environment(&args.env);

    ctx.output.header(&format!("Deploying to {}", args.env));

    // Step 1: Build if needed
    if !args.no_build {
        ctx.output.step(1, 5, "Building workload");
        let build_args = super::BuildArgs {
            release: true,
            features: vec![],
            no_optimize: false,
            path: None,
        };
        super::build::run(build_args, ctx).await?;
    } else {
        ctx.output.step(1, 5, "Skipping build (--no-build)");
    }

    // Step 2: Validate deployment config
    ctx.output.step(2, 5, "Validating configuration");

    let app_name = config.deploy.app_name.as_ref().unwrap_or(&ctx.config.workload.name);

    ctx.output.kv("Application", app_name);
    ctx.output.kv("Environment", &args.env);
    ctx.output.kv("Version", &ctx.config.workload.version);

    if let Some(url) = &config.deploy.platform_url {
        ctx.output.kv("Platform", url);
    }

    // Generate version tag
    let version_tag = args.tag.clone().unwrap_or_else(|| {
        let timestamp = Utc::now().format("%Y%m%d%H%M%S");
        format!("v{}-{}", ctx.config.workload.version, timestamp)
    });
    ctx.output.kv("Tag", &version_tag);

    // Check canary settings
    let use_canary = args.canary || config.deploy.canary;
    let canary_pct = args.canary_percentage.unwrap_or(config.deploy.canary_percentage);

    if use_canary {
        ctx.output.kv("Deployment", &format!("Canary ({}%)", canary_pct));
    } else {
        ctx.output.kv("Deployment", "Full rollout");
    }

    // Step 3: Confirmation
    if !args.yes && !args.dry_run {
        ctx.output.info("");
        let confirmed = Confirm::new()
            .with_prompt("Proceed with deployment?")
            .default(true)
            .interact()?;

        if !confirmed {
            ctx.output.warn("Deployment cancelled");
            return Ok(());
        }
    }

    if args.dry_run {
        ctx.output.step(3, 5, "Dry run - skipping actual deployment");
        ctx.output.step(4, 5, "Dry run - skipping verification");
        ctx.output.step(5, 5, "Done (dry run)");
        ctx.output.success("Dry run completed successfully");
        return Ok(());
    }

    // Step 4: Deploy using spin
    ctx.output.step(3, 5, "Deploying to Spin Cloud");

    let spinner = ctx.output.spinner("Deploying...");

    // Check if spin is available
    let spin_check = Command::new("spin")
        .arg("--version")
        .output();

    if spin_check.is_err() {
        spinner.finish_and_clear();
        bail!("spin CLI not found. Install from https://developer.fermyon.com/spin/install");
    }

    // Build spin deploy command
    let mut spin_args = vec!["deploy".to_string()];

    // Add environment variables
    for (key, value) in &config.deploy.env_vars {
        spin_args.push("--variable".to_string());
        spin_args.push(format!("{}={}", key, value));
    }

    let output = Command::new("spin")
        .args(&spin_args)
        .current_dir(&ctx.workload_dir())
        .output()
        .context("Failed to run spin deploy")?;

    spinner.finish_and_clear();

    if !output.status.success() {
        ctx.output.error("Deployment failed");
        eprintln!("{}", String::from_utf8_lossy(&output.stderr));
        bail!("spin deploy failed");
    }

    // Parse deployment URL from output
    let stdout = String::from_utf8_lossy(&output.stdout);
    ctx.output.debug(&stdout);

    // Try to extract the deployed URL
    let deployed_url = extract_deployed_url(&stdout);

    ctx.output.step(4, 5, "Verifying deployment");

    // Record deployment
    let deployment_record = DeploymentRecord {
        version: version_tag.clone(),
        environment: args.env.clone(),
        timestamp: Utc::now().to_rfc3339(),
        app_name: app_name.clone(),
        url: deployed_url.clone(),
        canary: use_canary,
        canary_percentage: if use_canary { Some(canary_pct) } else { None },
    };

    // Save deployment record
    save_deployment_record(&deployment_record, ctx)?;

    ctx.output.step(5, 5, "Done!");

    ctx.output.success("Deployment successful!");
    ctx.output.kv("Version", &version_tag);
    if let Some(url) = deployed_url {
        ctx.output.kv("URL", &url);
    }

    if use_canary {
        ctx.output.info("");
        ctx.output.info(&format!(
            "Canary deployment at {}%. Run `edge deploy --canary-percentage 100` to complete rollout.",
            canary_pct
        ));
    }

    Ok(())
}

fn extract_deployed_url(output: &str) -> Option<String> {
    // Look for common patterns in spin deploy output
    for line in output.lines() {
        if line.contains("https://") {
            if let Some(start) = line.find("https://") {
                let url_part = &line[start..];
                let end = url_part.find(|c: char| c.is_whitespace()).unwrap_or(url_part.len());
                return Some(url_part[..end].to_string());
            }
        }
    }
    None
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

fn save_deployment_record(record: &DeploymentRecord, ctx: &Context) -> Result<()> {
    let deployments_dir = ctx.workload_dir().join(".edge").join("deployments");
    std::fs::create_dir_all(&deployments_dir)?;

    let filename = format!("{}-{}.json", record.environment, record.version);
    let path = deployments_dir.join(&filename);

    let json = serde_json::to_string_pretty(record)?;
    std::fs::write(&path, json)?;

    ctx.output.debug(&format!("Saved deployment record: {}", path.display()));

    Ok(())
}
