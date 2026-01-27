//! Build a workload for deployment.

use std::process::Command;

use anyhow::{bail, Context as _, Result};

use super::BuildArgs;
use crate::context::Context;
use crate::output::format_bytes;

/// Run the build command.
pub async fn run(args: BuildArgs, ctx: &Context) -> Result<()> {
    let workload_dir = if let Some(ref path) = args.path {
        ctx.resolve_path(path)
    } else {
        ctx.workload_dir()
    };

    // Check for Cargo.toml
    if !workload_dir.join("Cargo.toml").exists() {
        bail!("No Cargo.toml found in {}. Is this a Rust project?", workload_dir.display());
    }

    ctx.output.header("Building workload");

    // Build arguments
    let mut cargo_args = vec!["build".to_string()];

    // Add target
    let target = &ctx.config.build.target;
    cargo_args.push("--target".to_string());
    cargo_args.push(target.clone());

    // Add profile
    if args.release || ctx.config.build.profile == "release" {
        cargo_args.push("--release".to_string());
    }

    // Add features
    let mut features = ctx.config.build.features.clone();
    features.extend(args.features.iter().cloned());
    if !features.is_empty() {
        cargo_args.push("--features".to_string());
        cargo_args.push(features.join(","));
    }

    // Add extra cargo args from config
    cargo_args.extend(ctx.config.build.cargo_args.clone());

    ctx.output.step(1, 3, "Running cargo build");
    ctx.output.debug(&format!("cargo {}", cargo_args.join(" ")));

    let spinner = ctx.output.spinner("Compiling...");

    let output = Command::new("cargo")
        .args(&cargo_args)
        .current_dir(&workload_dir)
        .output()
        .context("Failed to run cargo")?;

    spinner.finish_and_clear();

    if !output.status.success() {
        ctx.output.error("Build failed");
        eprintln!("{}", String::from_utf8_lossy(&output.stderr));
        bail!("cargo build failed");
    }

    ctx.output.success("Build completed");

    // Find the output WASM file
    let profile = if args.release || ctx.config.build.profile == "release" {
        "release"
    } else {
        "debug"
    };

    let wasm_dir = workload_dir.join("target").join(target).join(profile);

    // Find .wasm files
    let wasm_files: Vec<_> = std::fs::read_dir(&wasm_dir)?
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().map_or(false, |ext| ext == "wasm"))
        .collect();

    if wasm_files.is_empty() {
        bail!("No .wasm file found in {}", wasm_dir.display());
    }

    let wasm_path = wasm_files[0].path();
    let wasm_size = std::fs::metadata(&wasm_path)?.len();

    ctx.output.step(2, 3, &format!("Output: {}", wasm_path.display()));
    ctx.output.kv("Size", &format_bytes(wasm_size));

    // Optimize if requested
    if ctx.config.build.optimize && !args.no_optimize {
        ctx.output.step(3, 3, "Optimizing WASM");

        if let Ok(wasm_opt_path) = which_wasm_opt() {
            let spinner = ctx.output.spinner("Running wasm-opt...");

            let opt_output = wasm_path.with_extension("opt.wasm");
            let status = Command::new(&wasm_opt_path)
                .args(["-O3", "-o"])
                .arg(&opt_output)
                .arg(&wasm_path)
                .status();

            spinner.finish_and_clear();

            match status {
                Ok(s) if s.success() => {
                    let opt_size = std::fs::metadata(&opt_output)?.len();
                    let savings = wasm_size.saturating_sub(opt_size);
                    let savings_pct = (savings as f64 / wasm_size as f64) * 100.0;

                    // Replace original with optimized
                    std::fs::rename(&opt_output, &wasm_path)?;

                    ctx.output.success(&format!(
                        "Optimized: {} -> {} ({:.1}% smaller)",
                        format_bytes(wasm_size),
                        format_bytes(opt_size),
                        savings_pct
                    ));
                }
                _ => {
                    ctx.output.warn("wasm-opt failed, using unoptimized build");
                }
            }
        } else {
            ctx.output.debug("wasm-opt not found, skipping optimization");
            ctx.output.info("Tip: Install wasm-opt for smaller builds: cargo install wasm-opt");
        }
    } else {
        ctx.output.step(3, 3, "Skipping optimization");
    }

    ctx.output.success("Build complete!");
    ctx.output.kv("Output", &wasm_path.display().to_string());

    Ok(())
}

fn which_wasm_opt() -> Result<String> {
    // Try common locations
    for cmd in &["wasm-opt", "wasm-opt.exe"] {
        if Command::new(cmd).arg("--version").output().is_ok() {
            return Ok(cmd.to_string());
        }
    }
    bail!("wasm-opt not found")
}
