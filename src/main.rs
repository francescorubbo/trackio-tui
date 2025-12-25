//! trackio-tui: A Rust-based Terminal User Interface for trackio experiment visualization
//!
//! This is a drop-in replacement for `trackio show` providing a keyboard-driven
//! terminal dashboard for visualizing machine learning experiments.

mod cli;
mod data;
mod ui;
mod app;

use anyhow::Result;
use cli::{AppConfig, Cli, Commands};

fn main() -> Result<()> {
    // Parse CLI arguments
    let cli = Cli::parse_args();

    match cli.command {
        Commands::Show {
            project,
            theme,
            color_palette,
            interval,
            db_path,
        } => {
            let config = AppConfig::from_show_command(
                project,
                theme,
                color_palette,
                interval,
                db_path,
            );
            
            // Run the TUI application
            app::run(config)?;
        }
    }

    Ok(())
}
