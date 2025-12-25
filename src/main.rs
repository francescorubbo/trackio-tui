//! trackio-tui: A Rust-based Terminal User Interface for trackio experiment visualization
//!
//! This is a drop-in replacement for `trackio show` providing a keyboard-driven
//! terminal dashboard for visualizing machine learning experiments.

mod cli;
mod data;
mod ui;
mod app;

use anyhow::Result;
use cli::{AppConfig, Cli};

fn main() -> Result<()> {
    // Parse CLI arguments
    let cli = Cli::parse_args();
    let config = AppConfig::from_cli(&cli);
    
    // Run the TUI application
    app::run(config)?;

    Ok(())
}
