//! Data layer for accessing trackio's SQLite database.
//!
//! Handles loading projects, runs, metrics, and config from the local database.

mod models;
mod storage;

pub use models::{Config, Metric, Project, Run, RunStatus};
#[cfg(test)]
pub use models::MetricPoint;
pub use storage::Storage;

