//! Data layer for accessing trackio's SQLite database.
//!
//! Handles loading projects, runs, metrics, config, and comparison state.

mod comparison;
mod models;
mod storage;

pub use comparison::ComparisonState;
pub use models::{Config, Metric, Project, Run};
#[cfg(test)]
pub(crate) use models::MetricPoint;
pub use storage::Storage;
