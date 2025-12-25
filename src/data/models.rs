//! Data models representing trackio's experiment data.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Status of an experiment run
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum RunStatus {
    Running,
    Finished,
    Failed,
    Unknown,
}

impl RunStatus {
    #[allow(dead_code)]
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "running" | "live" => RunStatus::Running,
            "finished" | "done" | "completed" => RunStatus::Finished,
            "failed" | "error" => RunStatus::Failed,
            _ => RunStatus::Unknown,
        }
    }

    pub fn display(&self) -> &'static str {
        match self {
            RunStatus::Running => "live",
            RunStatus::Finished => "done",
            RunStatus::Failed => "fail",
            RunStatus::Unknown => "???",
        }
    }
}

/// A trackio project containing multiple runs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Project {
    pub name: String,
    pub run_count: usize,
    pub last_updated: Option<DateTime<Utc>>,
}

/// An individual experiment run within a project
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Run {
    pub id: String,
    pub project: String,
    pub name: Option<String>,
    pub status: RunStatus,
    pub created_at: Option<DateTime<Utc>>,
    pub finished_at: Option<DateTime<Utc>>,
    pub config: Vec<Config>,
}

impl Run {
    /// Get a display name for the run
    pub fn display_name(&self) -> String {
        self.name.clone().unwrap_or_else(|| {
            // Use first 8 chars of ID if no name
            if self.id.len() > 8 {
                format!("{}...", &self.id[..8])
            } else {
                self.id.clone()
            }
        })
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
            ConfigValue::Int(v) => write!(f, "{}", v),
            ConfigValue::Float(v) => {
                if v.abs() < 0.001 || v.abs() >= 1000.0 {
                    write!(f, "{:.2e}", v)
                } else {
                    write!(f, "{:.4}", v)
                }
            }
            ConfigValue::String(v) => write!(f, "{}", v),
            ConfigValue::Bool(v) => write!(f, "{}", v),
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

    /// Get the latest value for this metric
    #[allow(dead_code)]
    pub fn latest_value(&self) -> Option<f64> {
        self.points.last().map(|p| p.value)
    }

    /// Get min and max values for axis scaling
    #[allow(dead_code)]
    pub fn value_range(&self) -> Option<(f64, f64)> {
        if self.points.is_empty() {
            return None;
        }
        let mut min = f64::MAX;
        let mut max = f64::MIN;
        for p in &self.points {
            if p.value < min {
                min = p.value;
            }
            if p.value > max {
                max = p.value;
            }
        }
        Some((min, max))
    }

    /// Get step range
    pub fn step_range(&self) -> Option<(i64, i64)> {
        if self.points.is_empty() {
            return None;
        }
        let min = self.points.first().map(|p| p.step).unwrap_or(0);
        let max = self.points.last().map(|p| p.step).unwrap_or(0);
        Some((min, max))
    }

    /// Apply exponential moving average smoothing
    pub fn smoothed(&self, factor: f64) -> Vec<MetricPoint> {
        if self.points.is_empty() || factor <= 0.0 {
            return self.points.clone();
        }

        let alpha = 1.0 - factor.min(0.99);
        let mut smoothed = Vec::with_capacity(self.points.len());
        let mut ema = self.points[0].value;

        for point in &self.points {
            ema = alpha * point.value + (1.0 - alpha) * ema;
            smoothed.push(MetricPoint {
                step: point.step,
                value: ema,
                timestamp: point.timestamp,
            });
        }

        smoothed
    }
}

/// A single data point in a metric time series
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricPoint {
    pub step: i64,
    pub value: f64,
    pub timestamp: Option<DateTime<Utc>>,
}

