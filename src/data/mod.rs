//! Data layer for accessing trackio's SQLite database.
//!
//! Handles loading projects, runs, metrics, and config from the local database.

mod models;
mod storage;

pub use models::{Config, Metric, MetricPoint, Project, Run, RunStatus};
pub use storage::Storage;

