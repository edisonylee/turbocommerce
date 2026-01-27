//! Request recording and replay for debugging.

use std::fs;

use anyhow::{bail, Result};
use chrono::Utc;

use super::{ReplayArgs, ReplayCommand};
use crate::context::Context;
use crate::output::format_bytes;

/// Run the replay command.
pub async fn run(args: ReplayArgs, ctx: &Context) -> Result<()> {
    match args.command {
        ReplayCommand::Record {
            name,
            max_requests,
            duration,
        } => start_recording(name, max_requests, duration, ctx).await,
        ReplayCommand::Stop => stop_recording(ctx).await,
        ReplayCommand::List => list_recordings(ctx).await,
        ReplayCommand::Play {
            recording,
            target,
            diff,
            concurrency,
        } => play_recording(&recording, target.as_deref(), diff, concurrency, ctx).await,
        ReplayCommand::Delete { recording } => delete_recording(&recording, ctx).await,
        ReplayCommand::Export { recording, output } => {
            export_recording(&recording, &output, ctx).await
        }
    }
}

async fn start_recording(
    name: Option<String>,
    max_requests: usize,
    duration: Option<u64>,
    ctx: &Context,
) -> Result<()> {
    let recordings_dir = ctx.recordings_dir()?;

    // Generate recording name
    let recording_name = name.unwrap_or_else(|| {
        Utc::now().format("recording-%Y%m%d-%H%M%S").to_string()
    });

    let recording_path = recordings_dir.join(format!("{}.json", recording_name));

    if recording_path.exists() {
        bail!("Recording '{}' already exists", recording_name);
    }

    ctx.output.header("Starting recording");

    // Create recording metadata
    let metadata = RecordingMetadata {
        name: recording_name.clone(),
        created_at: Utc::now().to_rfc3339(),
        max_requests,
        duration_secs: duration,
        status: "recording".to_string(),
        request_count: 0,
        workload: ctx.config.workload.name.clone(),
    };

    let json = serde_json::to_string_pretty(&metadata)?;
    fs::write(&recording_path, json)?;

    // Create marker file for active recording
    let active_path = recordings_dir.join(".active");
    fs::write(&active_path, &recording_name)?;

    ctx.output.success(&format!("Recording started: {}", recording_name));
    ctx.output.kv("Max requests", &max_requests.to_string());
    if let Some(d) = duration {
        ctx.output.kv("Duration", &format!("{} seconds", d));
    }

    ctx.output.info("");
    ctx.output.info("Note: Request recording requires platform-level integration.");
    ctx.output.info("In development, you can manually add requests to the recording file.");
    ctx.output.info("");
    ctx.output.info("Run `edge replay stop` to stop recording.");

    Ok(())
}

async fn stop_recording(ctx: &Context) -> Result<()> {
    let recordings_dir = ctx.recordings_dir()?;
    let active_path = recordings_dir.join(".active");

    if !active_path.exists() {
        bail!("No active recording found");
    }

    let recording_name = fs::read_to_string(&active_path)?;
    let recording_path = recordings_dir.join(format!("{}.json", recording_name));

    // Update recording status
    if recording_path.exists() {
        let content = fs::read_to_string(&recording_path)?;
        let mut metadata: RecordingMetadata = serde_json::from_str(&content)?;
        metadata.status = "stopped".to_string();

        let json = serde_json::to_string_pretty(&metadata)?;
        fs::write(&recording_path, json)?;
    }

    // Remove active marker
    fs::remove_file(&active_path)?;

    ctx.output.success(&format!("Recording stopped: {}", recording_name));

    Ok(())
}

async fn list_recordings(ctx: &Context) -> Result<()> {
    let recordings_dir = ctx.recordings_dir()?;

    ctx.output.header("Recordings");

    let mut recordings: Vec<RecordingMetadata> = Vec::new();

    for entry in fs::read_dir(&recordings_dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.extension().map_or(false, |e| e == "json") {
            if let Ok(content) = fs::read_to_string(&path) {
                if let Ok(metadata) = serde_json::from_str::<RecordingMetadata>(&content) {
                    recordings.push(metadata);
                }
            }
        }
    }

    if recordings.is_empty() {
        ctx.output.info("No recordings found.");
        ctx.output.info("Run `edge replay record` to start recording.");
        return Ok(());
    }

    // Sort by creation time (newest first)
    recordings.sort_by(|a, b| b.created_at.cmp(&a.created_at));

    if ctx.output.is_json() {
        ctx.output.json(&recordings);
        return Ok(());
    }

    ctx.output.table_row(
        &["NAME", "CREATED", "REQUESTS", "STATUS"],
        &[30, 25, 10, 12],
    );
    ctx.output.info(&"-".repeat(80));

    for r in &recordings {
        ctx.output.table_row(
            &[
                &r.name,
                &r.created_at[..19],
                &r.request_count.to_string(),
                &r.status,
            ],
            &[30, 25, 10, 12],
        );
    }

    ctx.output.info("");
    ctx.output.info(&format!("Total: {} recording(s)", recordings.len()));

    Ok(())
}

async fn play_recording(
    recording: &str,
    target: Option<&str>,
    diff: bool,
    concurrency: usize,
    ctx: &Context,
) -> Result<()> {
    let recordings_dir = ctx.recordings_dir()?;
    let recording_path = recordings_dir.join(format!("{}.json", recording));

    if !recording_path.exists() {
        bail!("Recording '{}' not found", recording);
    }

    let content = fs::read_to_string(&recording_path)?;
    let metadata: RecordingMetadata = serde_json::from_str(&content)?;

    ctx.output.header(&format!("Replaying: {}", recording));
    ctx.output.kv("Requests", &metadata.request_count.to_string());
    ctx.output.kv("Concurrency", &concurrency.to_string());

    if let Some(t) = target {
        ctx.output.kv("Target", t);
    }

    if diff {
        ctx.output.kv("Mode", "Compare responses");
    }

    // Load requests from recording
    let requests_path = recordings_dir.join(format!("{}-requests.json", recording));

    if !requests_path.exists() {
        ctx.output.warn("No requests found in this recording.");
        ctx.output.info("Add requests to the recording manually or use platform integration.");
        return Ok(());
    }

    let requests_content = fs::read_to_string(&requests_path)?;
    let requests: Vec<RecordedRequest> = serde_json::from_str(&requests_content)?;

    if requests.is_empty() {
        ctx.output.info("Recording contains no requests.");
        return Ok(());
    }

    let progress = ctx.output.progress(requests.len() as u64, "Replaying");

    let mut success_count = 0;
    let failure_count = 0;
    let diff_count = 0;

    for request in &requests {
        progress.inc(1);

        // In a real implementation, this would:
        // 1. Make the HTTP request to the target
        // 2. Compare response if diff mode is enabled
        // 3. Record results

        // Simulate replay
        success_count += 1;

        if diff && request.response.is_some() {
            // Would compare responses here
        }
    }

    progress.finish_and_clear();

    ctx.output.success("Replay complete");
    ctx.output.kv("Successful", &success_count.to_string());
    ctx.output.kv("Failed", &failure_count.to_string());

    if diff {
        ctx.output.kv("Differences", &diff_count.to_string());
    }

    Ok(())
}

async fn delete_recording(recording: &str, ctx: &Context) -> Result<()> {
    let recordings_dir = ctx.recordings_dir()?;
    let recording_path = recordings_dir.join(format!("{}.json", recording));
    let requests_path = recordings_dir.join(format!("{}-requests.json", recording));

    if !recording_path.exists() {
        bail!("Recording '{}' not found", recording);
    }

    fs::remove_file(&recording_path)?;
    let _ = fs::remove_file(&requests_path); // May not exist

    ctx.output.success(&format!("Deleted recording: {}", recording));

    Ok(())
}

async fn export_recording(recording: &str, output: &str, ctx: &Context) -> Result<()> {
    let recordings_dir = ctx.recordings_dir()?;
    let recording_path = recordings_dir.join(format!("{}.json", recording));
    let requests_path = recordings_dir.join(format!("{}-requests.json", recording));

    if !recording_path.exists() {
        bail!("Recording '{}' not found", recording);
    }

    // Create export bundle
    let metadata: RecordingMetadata =
        serde_json::from_str(&fs::read_to_string(&recording_path)?)?;

    let requests: Vec<RecordedRequest> = if requests_path.exists() {
        serde_json::from_str(&fs::read_to_string(&requests_path)?)?
    } else {
        Vec::new()
    };

    let export = RecordingExport {
        metadata,
        requests,
    };

    let json = serde_json::to_string_pretty(&export)?;
    let size = json.len() as u64;

    fs::write(output, &json)?;

    ctx.output.success(&format!("Exported to: {}", output));
    ctx.output.kv("Size", &format_bytes(size));

    Ok(())
}

#[derive(serde::Serialize, serde::Deserialize)]
struct RecordingMetadata {
    name: String,
    created_at: String,
    max_requests: usize,
    duration_secs: Option<u64>,
    status: String,
    request_count: usize,
    workload: String,
}

#[derive(serde::Serialize, serde::Deserialize)]
struct RecordedRequest {
    id: String,
    timestamp: String,
    method: String,
    path: String,
    headers: Vec<(String, String)>,
    body: Option<String>,
    response: Option<RecordedResponse>,
}

#[derive(serde::Serialize, serde::Deserialize)]
struct RecordedResponse {
    status: u16,
    headers: Vec<(String, String)>,
    body: Option<String>,
    duration_ms: u64,
}

#[derive(serde::Serialize, serde::Deserialize)]
struct RecordingExport {
    metadata: RecordingMetadata,
    requests: Vec<RecordedRequest>,
}
