//! SQLite storage layer for reading trackio's experiment database.
//!
//! Trackio database schema:
//! - One .db file per project in ~/.cache/huggingface/trackio/
//! - `metrics` table: id, timestamp, run_name, step, metrics (JSON)
//! - `configs` table: id, run_name, config (JSON), created_at

use std::collections::HashMap;
use std::path::PathBuf;

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use rusqlite::{Connection, OpenFlags, Row};

use super::models::{Config, ConfigValue, Metric, MetricPoint, Project, Run};

/// Helper to read a column that might be stored as TEXT or BLOB
/// Trackio uses orjson which can write JSON as bytes (BLOB) rather than text
fn get_string_or_blob(row: &Row, idx: usize) -> rusqlite::Result<String> {
    // Try reading as string first
    match row.get::<_, String>(idx) {
        Ok(s) => Ok(s),
        Err(_) => {
            // Fall back to reading as blob and converting to string
            let blob: Vec<u8> = row.get(idx)?;
            String::from_utf8(blob)
                .map_err(|e| rusqlite::Error::FromSqlConversionFailure(
                    idx,
                    rusqlite::types::Type::Blob,
                    Box::new(e),
                ))
        }
    }
}

/// Storage interface for trackio's SQLite database
pub struct Storage {
    db_path: PathBuf,
}

impl Storage {
    /// Create a new Storage instance pointing to the trackio database directory
    pub fn new(db_path: PathBuf) -> Self {
        Storage { db_path }
    }

    /// Get the path to a project's database file
    fn project_db_path(&self, project: &str) -> PathBuf {
        self.db_path.join(format!("{project}.db"))
    }

    /// Open a read-only connection to a project database
    fn open_project_db(&self, project: &str) -> Result<Connection> {
        let path = self.project_db_path(project);
        if !path.exists() {
            anyhow::bail!("Project database not found: {path:?}");
        }
        Connection::open_with_flags(&path, OpenFlags::SQLITE_OPEN_READ_ONLY)
            .with_context(|| format!("Failed to open database: {path:?}"))
    }

    /// List all available projects by scanning for .db files
    pub fn list_projects(&self) -> Result<Vec<Project>> {
        let mut projects = Vec::new();

        if !self.db_path.exists() {
            return Ok(projects);
        }

        let entries = std::fs::read_dir(&self.db_path)
            .with_context(|| format!("Failed to read directory: {:?}", self.db_path))?;

        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().map(|e| e == "db").unwrap_or(false) {
                if let Some(name) = path.file_stem().and_then(|s| s.to_str()) {
                    // Skip auxiliary files
                    if name.starts_with('.') || name.ends_with("-shm") || name.ends_with("-wal") {
                        continue;
                    }

                    // Try to get run count and last updated from the database
                    let (run_count, last_updated) = self
                        .get_project_stats(name)
                        .unwrap_or((0, None));

                    projects.push(Project {
                        name: name.to_string(),
                        run_count,
                        last_updated,
                    });
                }
            }
        }

        // Sort by last updated (most recent first)
        projects.sort_by(|a, b| b.last_updated.cmp(&a.last_updated));

        Ok(projects)
    }

    /// Get statistics for a project (run count, last updated)
    fn get_project_stats(&self, project: &str) -> Result<(usize, Option<DateTime<Utc>>)> {
        let conn = self.open_project_db(project)?;

        // Get distinct run count from configs table
        let run_count: usize = conn
            .query_row("SELECT COUNT(DISTINCT run_name) FROM configs", [], |row| row.get(0))
            .unwrap_or(0);

        // Get last updated timestamp from configs
        let last_updated: Option<String> = conn
            .query_row(
                "SELECT MAX(created_at) FROM configs",
                [],
                |row| row.get(0),
            )
            .ok();

        let last_updated = last_updated.and_then(|ts| {
            DateTime::parse_from_rfc3339(&ts)
                .or_else(|_| DateTime::parse_from_str(&ts, "%Y-%m-%dT%H:%M:%S%.f"))
                .map(|dt| dt.with_timezone(&Utc))
                .ok()
        });

        Ok((run_count, last_updated))
    }

    /// List all runs for a project
    pub fn list_runs(&self, project: &str) -> Result<Vec<Run>> {
        let conn = self.open_project_db(project)?;
        let mut runs = Vec::new();

        // Query configs table for run information
        let mut stmt = conn.prepare(
            "SELECT run_name, config, created_at FROM configs ORDER BY created_at DESC"
        )?;

        let run_iter = stmt.query_map([], |row| {
            let run_name: String = row.get(0)?;
            // Config can be stored as TEXT or BLOB depending on how trackio wrote it
            let config_json: String = get_string_or_blob(row, 1)?;
            let created_at: String = row.get(2)?;
            Ok((run_name, config_json, created_at))
        })?;

        for run_result in run_iter {
            let (run_name, config_json, created_at) = run_result?;

            let config = parse_config_json(&config_json).unwrap_or_default();
            
            let created_at = DateTime::parse_from_rfc3339(&created_at)
                .or_else(|_| DateTime::parse_from_str(&created_at, "%Y-%m-%dT%H:%M:%S%.f"))
                .map(|dt| dt.with_timezone(&Utc))
                .ok();

            runs.push(Run {
                id: run_name.clone(),
                project: project.to_string(),
                name: Some(run_name),
                created_at,
                config,
            });
        }

        Ok(runs)
    }

    /// Get all metric names for a run
    pub fn list_metrics(&self, project: &str, run_id: &str) -> Result<Vec<String>> {
        let conn = self.open_project_db(project)?;

        // Get a sample metrics JSON to extract available metric names
        let metrics_json: Option<String> = conn
            .query_row(
                "SELECT metrics FROM metrics WHERE run_name = ? LIMIT 1",
                [run_id],
                |row| get_string_or_blob(row, 0),
            )
            .ok();

        if let Some(json) = metrics_json {
            if let Ok(map) = serde_json::from_str::<HashMap<String, serde_json::Value>>(&json) {
                let mut names: Vec<String> = map.keys().cloned().collect();
                names.sort();
                return Ok(names);
            }
        }

        Ok(Vec::new())
    }

    /// Get metric data for a specific run and metric name
    pub fn get_metric(&self, project: &str, run_id: &str, metric_name: &str) -> Result<Metric> {
        let conn = self.open_project_db(project)?;

        let mut stmt = conn.prepare(
            "SELECT step, metrics, timestamp FROM metrics WHERE run_name = ? ORDER BY step"
        )?;

        let row_iter = stmt.query_map([run_id], |row| {
            let step: i64 = row.get(0)?;
            let metrics_json: String = get_string_or_blob(row, 1)?;
            let timestamp: Option<String> = row.get(2)?;
            Ok((step, metrics_json, timestamp))
        })?;

        let mut metric = Metric::new(metric_name.to_string());

        for row in row_iter {
            let (step, metrics_json, timestamp) = row?;
            
            // Parse the JSON and extract the specific metric
            if let Ok(map) = serde_json::from_str::<HashMap<String, serde_json::Value>>(&metrics_json) {
                if let Some(value) = map.get(metric_name) {
                    if let Some(v) = value.as_f64() {
                        let ts = timestamp.and_then(|t| {
                            DateTime::parse_from_rfc3339(&t)
                                .or_else(|_| DateTime::parse_from_str(&t, "%Y-%m-%dT%H:%M:%S%.f"))
                                .map(|dt| dt.with_timezone(&Utc))
                                .ok()
                        });
                        
                        metric.points.push(MetricPoint {
                            step,
                            value: v,
                            timestamp: ts,
                        });
                    }
                }
            }
        }

        Ok(metric)
    }

    /// Get all metrics for a run
    pub fn get_all_metrics(&self, project: &str, run_id: &str) -> Result<Vec<Metric>> {
        let metric_names = self.list_metrics(project, run_id)?;
        let mut metrics = Vec::new();

        for name in metric_names {
            let metric = self.get_metric(project, run_id, &name)?;
            if !metric.points.is_empty() {
                metrics.push(metric);
            }
        }

        Ok(metrics)
    }

}

/// Parse JSON config string into Config vector
fn parse_config_json(json: &str) -> Result<Vec<Config>> {
    let map: HashMap<String, serde_json::Value> = serde_json::from_str(json)?;
    
    let mut configs: Vec<Config> = map
        .into_iter()
        .filter(|(key, _)| !key.starts_with('_')) // Skip internal fields like _Username, _Created
        .map(|(key, value)| Config {
            key,
            value: json_to_config_value(value),
        })
        .collect();

    // Sort by key for consistent display
    configs.sort_by(|a, b| a.key.cmp(&b.key));

    Ok(configs)
}

/// Convert JSON value to ConfigValue
fn json_to_config_value(value: serde_json::Value) -> ConfigValue {
    match value {
        serde_json::Value::Null => ConfigValue::Null,
        serde_json::Value::Bool(b) => ConfigValue::Bool(b),
        serde_json::Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                ConfigValue::Int(i)
            } else if let Some(f) = n.as_f64() {
                ConfigValue::Float(f)
            } else {
                ConfigValue::String(n.to_string())
            }
        }
        serde_json::Value::String(s) => ConfigValue::String(s),
        serde_json::Value::Array(arr) => {
            ConfigValue::String(serde_json::to_string(&arr).unwrap_or_default())
        }
        serde_json::Value::Object(obj) => {
            ConfigValue::String(serde_json::to_string(&obj).unwrap_or_default())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_config_json() {
        let json = r#"{"epochs": 10, "learning_rate": 0.001, "name": "test", "_Created": "2025-01-01"}"#;
        let configs = parse_config_json(json).unwrap();
        // Should have 3 items (excluding _Created)
        assert_eq!(configs.len(), 3);
    }

    #[test]
    fn test_json_to_config_value() {
        assert!(matches!(
            json_to_config_value(serde_json::json!(42)),
            ConfigValue::Int(42)
        ));
        assert!(matches!(
            json_to_config_value(serde_json::json!(std::f64::consts::PI)),
            ConfigValue::Float(_)
        ));
        assert!(matches!(
            json_to_config_value(serde_json::json!("hello")),
            ConfigValue::String(_)
        ));
    }
}
