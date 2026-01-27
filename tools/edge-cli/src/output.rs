//! Output formatting for the CLI.

use console::{style, Term};
use indicatif::{ProgressBar, ProgressStyle};

/// Output handler for CLI messages.
#[derive(Clone)]
pub struct Output {
    verbose: bool,
    json: bool,
    term: Term,
}

impl Output {
    /// Create a new output handler.
    pub fn new(verbose: bool, json: bool) -> Self {
        Self {
            verbose,
            json,
            term: Term::stderr(),
        }
    }

    /// Print an info message.
    pub fn info(&self, msg: &str) {
        if self.json {
            return;
        }
        println!("{} {}", style("ℹ").blue(), msg);
    }

    /// Print a success message.
    pub fn success(&self, msg: &str) {
        if self.json {
            return;
        }
        println!("{} {}", style("✓").green(), msg);
    }

    /// Print a warning message.
    pub fn warn(&self, msg: &str) {
        if self.json {
            return;
        }
        eprintln!("{} {}", style("⚠").yellow(), msg);
    }

    /// Print an error message.
    pub fn error(&self, msg: &str) {
        if self.json {
            eprintln!(r#"{{"error": "{}"}}"#, msg.replace('"', "\\\""));
            return;
        }
        eprintln!("{} {}", style("✗").red(), style(msg).red());
    }

    /// Print a debug message (only in verbose mode).
    pub fn debug(&self, msg: &str) {
        if !self.verbose || self.json {
            return;
        }
        eprintln!("{} {}", style("→").dim(), style(msg).dim());
    }

    /// Print a header/title.
    pub fn header(&self, msg: &str) {
        if self.json {
            return;
        }
        println!("\n{}", style(msg).bold().underlined());
    }

    /// Print a step in a process.
    pub fn step(&self, num: usize, total: usize, msg: &str) {
        if self.json {
            return;
        }
        println!(
            "{} {}",
            style(format!("[{}/{}]", num, total)).dim(),
            msg
        );
    }

    /// Print JSON output.
    pub fn json<T: serde::Serialize>(&self, value: &T) {
        if let Ok(json) = serde_json::to_string_pretty(value) {
            println!("{}", json);
        }
    }

    /// Print a key-value pair.
    pub fn kv(&self, key: &str, value: &str) {
        if self.json {
            return;
        }
        println!("  {}: {}", style(key).dim(), value);
    }

    /// Print a list item.
    pub fn list_item(&self, item: &str) {
        if self.json {
            return;
        }
        println!("  {} {}", style("•").dim(), item);
    }

    /// Print a table row.
    pub fn table_row(&self, cols: &[&str], widths: &[usize]) {
        if self.json {
            return;
        }
        let formatted: Vec<String> = cols
            .iter()
            .zip(widths.iter())
            .map(|(col, width)| format!("{:width$}", col, width = width))
            .collect();
        println!("  {}", formatted.join("  "));
    }

    /// Create a progress bar.
    pub fn progress(&self, len: u64, msg: &str) -> ProgressBar {
        if self.json {
            return ProgressBar::hidden();
        }

        let pb = ProgressBar::new(len);
        pb.set_style(
            ProgressStyle::default_bar()
                .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} {msg}")
                .unwrap()
                .progress_chars("#>-"),
        );
        pb.set_message(msg.to_string());
        pb
    }

    /// Create a spinner for indeterminate progress.
    pub fn spinner(&self, msg: &str) -> ProgressBar {
        if self.json {
            return ProgressBar::hidden();
        }

        let pb = ProgressBar::new_spinner();
        pb.set_style(
            ProgressStyle::default_spinner()
                .template("{spinner:.green} {msg}")
                .unwrap(),
        );
        pb.set_message(msg.to_string());
        pb.enable_steady_tick(std::time::Duration::from_millis(100));
        pb
    }

    /// Check if verbose mode is enabled.
    pub fn is_verbose(&self) -> bool {
        self.verbose
    }

    /// Check if JSON mode is enabled.
    pub fn is_json(&self) -> bool {
        self.json
    }

    /// Get terminal width.
    pub fn term_width(&self) -> usize {
        self.term.size().1 as usize
    }
}

/// Status badge for deployment states.
pub fn status_badge(status: &str) -> String {
    match status.to_lowercase().as_str() {
        "deployed" | "active" | "healthy" => style(status).green().to_string(),
        "deploying" | "pending" => style(status).yellow().to_string(),
        "failed" | "error" => style(status).red().to_string(),
        "inactive" | "stopped" => style(status).dim().to_string(),
        _ => status.to_string(),
    }
}

/// Format bytes as human-readable size.
pub fn format_bytes(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;

    if bytes >= GB {
        format!("{:.2} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.2} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.2} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} B", bytes)
    }
}

/// Format duration as human-readable string.
pub fn format_duration(secs: u64) -> String {
    if secs < 60 {
        format!("{}s", secs)
    } else if secs < 3600 {
        format!("{}m {}s", secs / 60, secs % 60)
    } else {
        format!("{}h {}m", secs / 3600, (secs % 3600) / 60)
    }
}
