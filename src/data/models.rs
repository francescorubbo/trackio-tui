//! Data models representing trackio's experiment data.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// A trackio project containing multiple runs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Project {
    pub name: String,
    pub run_count: usize,
    pub last_updated: Option<DateTime<Utc>>,
}

/// Maximum characters to show in display name before truncating
const DISPLAY_NAME_MAX_LEN: usize = 8;

/// An individual experiment run within a project
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Run {
    pub id: String,
    pub project: String,
    pub created_at: Option<DateTime<Utc>>,
    pub config: Vec<Config>,
    /// Cached display name (computed on construction, derived from id)
    pub display_name: String,
}

impl Run {
    /// Create a new Run with cached display_name derived from id
    pub fn new(
        id: String,
        project: String,
        created_at: Option<DateTime<Utc>>,
        config: Vec<Config>,
    ) -> Self {
        // Truncate long IDs for display
        let display_name = if id.len() > DISPLAY_NAME_MAX_LEN {
            format!("{}...", &id[..DISPLAY_NAME_MAX_LEN])
        } else {
            id.clone()
        };
        Run {
            id,
            project,
            created_at,
            config,
            display_name,
        }
    }
}

/// A configuration key-value pair for a run
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub key: String,
    pub value: ConfigValue,
}

/// Possible types for config values
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ConfigValue {
    Int(i64),
    Float(f64),
    String(String),
    Bool(bool),
    Null,
}

impl std::fmt::Display for ConfigValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConfigValue::Int(v) => write!(f, "{v}"),
            ConfigValue::Float(v) => {
                if v.abs() < 0.001 || v.abs() >= 1000.0 {
                    write!(f, "{v:.2e}")
                } else {
                    write!(f, "{v:.4}")
                }
            }
            ConfigValue::String(v) => write!(f, "{v}"),
            ConfigValue::Bool(v) => write!(f, "{v}"),
            ConfigValue::Null => write!(f, "null"),
        }
    }
}

/// A metric tracked during training (e.g., "train_loss", "accuracy")
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Metric {
    pub name: String,
    pub points: Vec<MetricPoint>,
}

impl Metric {
    pub fn new(name: String) -> Self {
        Metric {
            name,
            points: Vec::new(),
        }
    }
}

/// A single data point in a metric time series
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricPoint {
    pub step: i64,
    pub value: f64,
    pub timestamp: Option<DateTime<Utc>>,
}
