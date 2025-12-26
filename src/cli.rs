//! Command-line interface argument parsing for trackio-tui.
//!
//! Provides CLI for launching the TUI dashboard:
//! - `trackio-tui --project "my-project"`
//! - `trackio-tui --interval 5`

use clap::Parser;

/// A Rust-based Terminal User Interface for visualizing trackio experiments.
///
/// Drop-in replacement for `trackio show` with keyboard-driven navigation.
#[derive(Parser, Debug)]
#[command(name = "trackio-tui")]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    /// Name of the project to display
    #[arg(short, long)]
    pub project: Option<String>,

    /// Update interval in seconds for live refresh
    #[arg(short, long, default_value = "2")]
    pub interval: u64,

    /// Path to the trackio database directory
    /// Defaults to ~/.cache/huggingface/trackio/
    #[arg(long)]
    pub db_path: Option<String>,
}

impl Cli {
    /// Parse command line arguments
    pub fn parse_args() -> Self {
        Cli::parse()
    }
}

/// Configuration derived from CLI arguments
#[derive(Debug, Clone)]
pub struct AppConfig {
    pub project: Option<String>,
    pub refresh_interval_secs: u64,
    pub db_path: std::path::PathBuf,
}

impl AppConfig {
    /// Create AppConfig from CLI arguments
    pub fn from_cli(cli: &Cli) -> Self {
        // Determine database path
        let db_path = cli
            .db_path
            .as_ref()
            .map(std::path::PathBuf::from)
            .unwrap_or_else(|| {
                // Check TRACKIO_DIR environment variable first
                if let Ok(trackio_dir) = std::env::var("TRACKIO_DIR") {
                    std::path::PathBuf::from(trackio_dir)
                } else {
                    // Default to ~/.cache/huggingface/trackio/
                    dirs::home_dir()
                        .unwrap_or_else(|| std::path::PathBuf::from("."))
                        .join(".cache")
                        .join("huggingface")
                        .join("trackio")
                }
            });

        AppConfig {
            project: cli.project.clone(),
            refresh_interval_secs: cli.interval,
            db_path,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let cli = Cli {
            project: None,
            interval: 2,
            db_path: None,
        };
        let config = AppConfig::from_cli(&cli);
        assert_eq!(config.refresh_interval_secs, 2);
    }
}
