//! Initialize a new workload project.

use std::path::Path;

use anyhow::{bail, Context as _, Result};

use super::InitArgs;
use crate::config::generate_default_config;
use crate::context::Context;

/// Run the init command.
pub async fn run(args: InitArgs, ctx: &Context) -> Result<()> {
    let name = if args.name == "." {
        // Use current directory name
        ctx.cwd
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("my-workload")
            .to_string()
    } else {
        args.name.clone()
    };

    ctx.output.header(&format!("Initializing workload: {}", name));

    let target_dir = if args.name == "." {
        ctx.cwd.clone()
    } else {
        ctx.cwd.join(&args.name)
    };

    // Check if directory exists and is not empty
    if target_dir.exists() && target_dir.read_dir()?.next().is_some() && args.name != "." {
        bail!("Directory '{}' already exists and is not empty", args.name);
    }

    // Create directory if needed
    if !target_dir.exists() {
        std::fs::create_dir_all(&target_dir)?;
        ctx.output.debug(&format!("Created directory: {}", target_dir.display()));
    }

    // Generate files based on template
    match args.template.as_str() {
        "basic" => generate_basic_template(&target_dir, &name, ctx)?,
        "streaming" => generate_streaming_template(&target_dir, &name, ctx)?,
        _ => bail!("Unknown template: {}. Available: basic, streaming", args.template),
    }

    // Initialize git
    if !args.no_git && !target_dir.join(".git").exists() {
        ctx.output.step(4, 5, "Initializing git repository");
        let status = std::process::Command::new("git")
            .args(["init"])
            .current_dir(&target_dir)
            .status()
            .context("Failed to run git init")?;

        if !status.success() {
            ctx.output.warn("Git initialization failed, continuing anyway");
        }
    }

    ctx.output.step(5, 5, "Done!");
    ctx.output.success(&format!("Workload '{}' initialized successfully", name));
    ctx.output.info("");
    ctx.output.info("Next steps:");
    ctx.output.list_item(&format!("cd {}", if args.name == "." { "." } else { &args.name }));
    ctx.output.list_item("edge build");
    ctx.output.list_item("spin up  # to test locally");
    ctx.output.list_item("edge deploy");

    Ok(())
}

fn generate_basic_template(dir: &Path, name: &str, ctx: &Context) -> Result<()> {
    // Step 1: Create Cargo.toml
    ctx.output.step(1, 5, "Creating Cargo.toml");
    let cargo_toml = format!(
        r#"[package]
name = "{name}"
version = "0.1.0"
edition = "2021"

[lib]
crate-type = ["cdylib"]

[dependencies]
spin-sdk = "5.0"
anyhow = "1"
serde = {{ version = "1", features = ["derive"] }}
serde_json = "1"
"#,
        name = name.replace('-', "_")
    );
    std::fs::write(dir.join("Cargo.toml"), cargo_toml)?;

    // Step 2: Create spin.toml
    ctx.output.step(2, 5, "Creating spin.toml");
    let spin_toml = format!(
        r#"spin_manifest_version = 2

[application]
name = "{name}"
version = "0.1.0"
authors = []
description = "An edge streaming SSR workload"

[[trigger.http]]
route = "/hello"
component = "{name}"

[component.{name}]
source = "target/wasm32-wasip1/release/{name_underscore}.wasm"
allowed_outbound_hosts = ["https://*"]

[component.{name}.build]
command = "cargo build --target wasm32-wasip1 --release"
"#,
        name = name,
        name_underscore = name.replace('-', "_")
    );
    std::fs::write(dir.join("spin.toml"), spin_toml)?;

    // Step 3: Create edge.toml
    ctx.output.step(3, 5, "Creating edge.toml");
    let edge_toml = generate_default_config(name);
    std::fs::write(dir.join("edge.toml"), edge_toml)?;

    // Step 4: Create src/lib.rs
    std::fs::create_dir_all(dir.join("src"))?;
    let lib_rs = r#"use spin_sdk::http::{IncomingRequest, OutgoingResponse, ResponseOutparam, Fields};
use spin_sdk::http_component;

#[http_component]
async fn handle(_req: IncomingRequest, response_out: ResponseOutparam) {
    let headers = Fields::from_list(&[
        ("content-type".to_owned(), "text/html".into()),
    ]).unwrap();

    let response = OutgoingResponse::new(headers);
    response.set_status_code(200).unwrap();

    let body = response.take_body();
    response_out.set(response);

    use futures::SinkExt;
    let mut body = body;
    let _ = body.send(b"<h1>Hello from the edge!</h1>".to_vec()).await;
}
"#;
    std::fs::write(dir.join("src/lib.rs"), lib_rs)?;

    // Create .gitignore
    let gitignore = r#"/target
Cargo.lock
.spin/
*.wasm
"#;
    std::fs::write(dir.join(".gitignore"), gitignore)?;

    Ok(())
}

fn generate_streaming_template(dir: &Path, name: &str, ctx: &Context) -> Result<()> {
    // Similar to basic but with streaming SSR example
    ctx.output.step(1, 5, "Creating Cargo.toml");
    let cargo_toml = format!(
        r#"[package]
name = "{name}"
version = "0.1.0"
edition = "2021"

[lib]
crate-type = ["cdylib"]

[dependencies]
spin-sdk = "5.0"
anyhow = "1"
serde = {{ version = "1", features = ["derive"] }}
serde_json = "1"
futures = "0.3"
"#,
        name = name.replace('-', "_")
    );
    std::fs::write(dir.join("Cargo.toml"), cargo_toml)?;

    ctx.output.step(2, 5, "Creating spin.toml");
    let spin_toml = format!(
        r#"spin_manifest_version = 2

[application]
name = "{name}"
version = "0.1.0"
description = "A streaming SSR workload"

[[trigger.http]]
route = "/..."
component = "{name}"

[component.{name}]
source = "target/wasm32-wasip1/release/{name_underscore}.wasm"
allowed_outbound_hosts = ["https://*"]

[component.{name}.build]
command = "cargo build --target wasm32-wasip1 --release"
"#,
        name = name,
        name_underscore = name.replace('-', "_")
    );
    std::fs::write(dir.join("spin.toml"), spin_toml)?;

    ctx.output.step(3, 5, "Creating edge.toml");
    let edge_toml = generate_default_config(name);
    std::fs::write(dir.join("edge.toml"), edge_toml)?;

    std::fs::create_dir_all(dir.join("src"))?;
    let lib_rs = r##"//! Streaming SSR workload example.

use futures::SinkExt;
use spin_sdk::http::{Fields, IncomingRequest, OutgoingResponse, ResponseOutparam};
use spin_sdk::http_component;

#[http_component]
async fn handle(_req: IncomingRequest, response_out: ResponseOutparam) {
    let headers = Fields::from_list(&[
        ("content-type".to_owned(), "text/html; charset=utf-8".into()),
    ])
    .unwrap();

    let response = OutgoingResponse::new(headers);
    response.set_status_code(200).unwrap();

    let body = response.take_body();
    response_out.set(response);
    let mut body = body;

    // Send shell immediately
    let shell = r#"<!DOCTYPE html>
<html>
<head>
    <title>Streaming SSR</title>
    <style>
        body { font-family: system-ui; max-width: 800px; margin: 0 auto; padding: 20px; }
        .loading { color: #666; }
        section { margin: 20px 0; padding: 20px; border: 1px solid #eee; border-radius: 8px; }
    </style>
</head>
<body>
    <h1>Streaming SSR Demo</h1>
    <p>Shell flushed! Loading content...</p>
"#;
    let _ = body.send(shell.as_bytes().to_vec()).await;

    // Simulate async data loading and stream sections
    let section1 = r#"
    <section>
        <h2>Section 1</h2>
        <p>This content was streamed from the edge!</p>
    </section>
"#;
    let _ = body.send(section1.as_bytes().to_vec()).await;

    let section2 = r#"
    <section>
        <h2>Section 2</h2>
        <p>More content, also streamed.</p>
    </section>
"#;
    let _ = body.send(section2.as_bytes().to_vec()).await;

    // Close the document
    let closing = r#"
    <p><em>All sections loaded!</em></p>
</body>
</html>
"#;
    let _ = body.send(closing.as_bytes().to_vec()).await;
}
"##;
    std::fs::write(dir.join("src").join("lib.rs"), lib_rs)?;

    let gitignore = "/target\nCargo.lock\n.spin/\n*.wasm\n";
    std::fs::write(dir.join(".gitignore"), gitignore)?;

    Ok(())
}
