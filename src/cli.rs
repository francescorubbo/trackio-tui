//! Command-line interface argument parsing for trackio-tui.
//!
//! Provides CLI syntax compatible with `trackio show`:
//! - `trackio-tui show --project "my-project"`
//! - `trackio-tui show --theme "soft"`
//! - `trackio-tui show --color-palette "#FF0000,#00FF00,#0000FF"`

use clap::{Parser, Subcommand};

/// A Rust-based Terminal User Interface for visualizing trackio experiments.
///
/// Drop-in replacement for `trackio show` with keyboard-driven navigation.
#[derive(Parser, Debug)]
#[command(name = "trackio-tui")]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Launch the TUI dashboard to visualize experiments
    Show {
        /// Name of the project to display
        #[arg(short, long)]
        project: Option<String>,

        /// Theme for the dashboard (e.g., "soft", "dark")
        #[arg(short, long)]
        theme: Option<String>,

        /// Comma-separated hex color palette for plot lines
        /// Example: "#FF0000,#00FF00,#0000FF"
        #[arg(short, long)]
        color_palette: Option<String>,

        /// Update interval in seconds for live refresh
        #[arg(short, long, default_value = "2")]
        interval: u64,

        /// Path to the trackio database directory
        /// Defaults to ~/.cache/huggingface/trackio/
        #[arg(long)]
        db_path: Option<String>,
    },
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
    pub theme: String,
    pub color_palette: Vec<String>,
    pub refresh_interval_secs: u64,
    pub db_path: std::path::PathBuf,
}

impl AppConfig {
    /// Create AppConfig from CLI Commands
    pub fn from_show_command(
        project: Option<String>,
        theme: Option<String>,
        color_palette: Option<String>,
        interval: u64,
        db_path: Option<String>,
    ) -> Self {
        // Parse color palette
        let colors = color_palette
            .map(|p| p.split(',').map(|s| s.trim().to_string()).collect())
            .unwrap_or_else(|| {
                // Default color palette
                vec![
                    "#FF6B6B".to_string(), // Red
                    "#4ECDC4".to_string(), // Teal
                    "#45B7D1".to_string(), // Blue
                    "#96CEB4".to_string(), // Green
                    "#FFEAA7".to_string(), // Yellow
                    "#DDA0DD".to_string(), // Plum
                    "#98D8C8".to_string(), // Mint
                    "#F7DC6F".to_string(), // Gold
                ]
            });

        // Determine database path
        let db_path = db_path
            .map(std::path::PathBuf::from)
            .unwrap_or_else(|| {
                // Check TRACKIO_DIR environment variable first
                if let Ok(trackio_dir) = std::env::var("TRACKIO_DIR") {
                    std::path::PathBuf::from(trackio_dir)
                } else {
                    // Default to ~/.cache/huggingface/trackio/
                    // Note: trackio uses this path on all platforms, not the system cache dir
                    dirs::home_dir()
                        .unwrap_or_else(|| std::path::PathBuf::from("."))
                        .join(".cache")
                        .join("huggingface")
                        .join("trackio")
                }
            });

        AppConfig {
            project,
            theme: theme.unwrap_or_else(|| "default".to_string()),
            color_palette: colors,
            refresh_interval_secs: interval,
            db_path,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = AppConfig::from_show_command(None, None, None, 2, None);
        assert_eq!(config.theme, "default");
        assert_eq!(config.refresh_interval_secs, 2);
        assert!(!config.color_palette.is_empty());
    }

    #[test]
    fn test_custom_colors() {
        let config = AppConfig::from_show_command(
            None,
            None,
            Some("#FF0000,#00FF00".to_string()),
            2,
            None,
        );
        assert_eq!(config.color_palette.len(), 2);
        assert_eq!(config.color_palette[0], "#FF0000");
    }
}

