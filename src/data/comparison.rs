//! Run comparison state management.
//!
//! Manages which runs are marked for comparison and caches their metrics data.
//! Uses run IDs (strings) instead of indices for robustness across data refreshes.

use std::collections::{HashMap, HashSet};

use super::Metric;

/// Manages run comparison state and cached metrics
#[derive(Debug, Default)]
pub struct ComparisonState {
    /// Run IDs marked for comparison (HashSet for O(1) lookup)
    marked_runs: HashSet<String>,
    /// Cached metrics for comparison, keyed by run ID
    metrics_cache: HashMap<String, Vec<Metric>>,
}

impl ComparisonState {
    /// Create a new empty comparison state
    pub fn new() -> Self {
        Self::default()
    }

    /// Toggle a run's comparison status. Returns true if run is now marked.
    pub fn toggle_run(&mut self, run_id: &str) -> bool {
        if self.marked_runs.contains(run_id) {
            self.marked_runs.remove(run_id);
            self.metrics_cache.remove(run_id);
            false
        } else {
            self.marked_runs.insert(run_id.to_string());
            true
        }
    }

    /// Check if a run is marked for comparison
    #[allow(dead_code)] // Used in tests
    pub fn is_marked(&self, run_id: &str) -> bool {
        self.marked_runs.contains(run_id)
    }

    /// Get the set of marked run IDs
    pub fn marked_run_ids(&self) -> &HashSet<String> {
        &self.marked_runs
    }

    /// Clear all comparison state
    pub fn clear(&mut self) {
        self.marked_runs.clear();
        self.metrics_cache.clear();
    }

    /// Cache metrics for a run
    pub fn cache_metrics(&mut self, run_id: &str, metrics: Vec<Metric>) {
        self.metrics_cache.insert(run_id.to_string(), metrics);
    }

    /// Get cached metrics for a specific run
    #[allow(dead_code)] // Used in tests
    pub fn get_cached_metrics(&self, run_id: &str) -> Option<&Vec<Metric>> {
        self.metrics_cache.get(run_id)
    }

    /// Get comparison metrics for display, excluding the currently selected run.
    /// Returns an iterator of (run_id, metric) pairs.
    pub fn get_comparison_metrics<'a>(
        &'a self,
        selected_run_id: &'a str,
    ) -> impl Iterator<Item = (&'a str, &'a Metric)> + 'a {
        self.marked_runs
            .iter()
            .filter(move |run_id| run_id.as_str() != selected_run_id)
            .flat_map(|run_id| {
                self.metrics_cache
                    .get(run_id)
                    .into_iter()
                    .flat_map(move |metrics| metrics.iter().map(move |m| (run_id.as_str(), m)))
            })
    }

    /// Check if there are any runs marked for comparison
    #[allow(dead_code)] // Used in tests
    pub fn has_comparisons(&self) -> bool {
        !self.marked_runs.is_empty()
    }

    /// Remove runs that are no longer valid (not in the provided list of valid IDs)
    pub fn prune_invalid_runs(&mut self, valid_run_ids: &HashSet<String>) {
        self.marked_runs
            .retain(|run_id| valid_run_ids.contains(run_id));
        self.metrics_cache
            .retain(|run_id, _| valid_run_ids.contains(run_id));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::MetricPoint;

    fn make_metric(name: &str, values: &[f64]) -> Metric {
        Metric {
            name: name.to_string(),
            points: values
                .iter()
                .enumerate()
                .map(|(i, &v)| MetricPoint {
                    step: i as i64,
                    value: v,
                    timestamp: None,
                })
                .collect(),
        }
    }

    #[test]
    fn test_toggle_run_adds_and_removes() {
        let mut state = ComparisonState::new();

        // Initially empty
        assert!(!state.is_marked("run-0"));
        assert!(state.marked_run_ids().is_empty());

        // Toggle on
        let added = state.toggle_run("run-0");
        assert!(added);
        assert!(state.is_marked("run-0"));
        assert!(state.marked_run_ids().contains("run-0"));
        assert_eq!(state.marked_run_ids().len(), 1);

        // Toggle off
        let added = state.toggle_run("run-0");
        assert!(!added);
        assert!(!state.is_marked("run-0"));
        assert!(state.marked_run_ids().is_empty());
    }

    #[test]
    fn test_multiple_runs_marked() {
        let mut state = ComparisonState::new();

        state.toggle_run("run-1");
        state.toggle_run("run-3");
        state.toggle_run("run-5");

        assert!(!state.is_marked("run-0"));
        assert!(state.is_marked("run-1"));
        assert!(!state.is_marked("run-2"));
        assert!(state.is_marked("run-3"));
        assert!(!state.is_marked("run-4"));
        assert!(state.is_marked("run-5"));
        assert_eq!(state.marked_run_ids().len(), 3);
    }

    #[test]
    fn test_clear_removes_all() {
        let mut state = ComparisonState::new();

        state.toggle_run("run-0");
        state.toggle_run("run-1");
        state.cache_metrics("run-0", vec![make_metric("loss", &[1.0, 0.5])]);
        state.cache_metrics("run-1", vec![make_metric("loss", &[0.9, 0.4])]);

        assert!(state.has_comparisons());
        assert!(state.get_cached_metrics("run-0").is_some());

        state.clear();

        assert!(!state.has_comparisons());
        assert!(state.marked_run_ids().is_empty());
        assert!(state.get_cached_metrics("run-0").is_none());
        assert!(state.get_cached_metrics("run-1").is_none());
    }

    #[test]
    fn test_get_comparison_metrics_excludes_selected() {
        let mut state = ComparisonState::new();

        // Mark runs 0, 1, 2
        state.toggle_run("run-0");
        state.toggle_run("run-1");
        state.toggle_run("run-2");

        state.cache_metrics("run-0", vec![make_metric("loss", &[1.0])]);
        state.cache_metrics("run-1", vec![make_metric("loss", &[0.9])]);
        state.cache_metrics("run-2", vec![make_metric("loss", &[0.8])]);

        // When selected_run is "run-1", we should get metrics for runs 0 and 2 only
        let comparison: Vec<(&str, &Metric)> = state.get_comparison_metrics("run-1").collect();

        assert_eq!(comparison.len(), 2);
        assert!(comparison.iter().any(|(id, _)| *id == "run-0"));
        assert!(comparison.iter().any(|(id, _)| *id == "run-2"));
        assert!(!comparison.iter().any(|(id, _)| *id == "run-1"));
    }

    #[test]
    fn test_get_comparison_metrics_with_multiple_metrics_per_run() {
        let mut state = ComparisonState::new();

        state.toggle_run("run-0");
        state.cache_metrics(
            "run-0",
            vec![
                make_metric("loss", &[1.0, 0.5]),
                make_metric("accuracy", &[0.5, 0.8]),
            ],
        );

        // When selected is "run-1", we get all metrics from run-0
        let comparison: Vec<(&str, &Metric)> = state.get_comparison_metrics("run-1").collect();

        assert_eq!(comparison.len(), 2);
        assert!(comparison.iter().all(|(id, _)| *id == "run-0"));

        let names: Vec<&str> = comparison.iter().map(|(_, m)| m.name.as_str()).collect();
        assert!(names.contains(&"loss"));
        assert!(names.contains(&"accuracy"));
    }

    #[test]
    fn test_prune_invalid_runs() {
        let mut state = ComparisonState::new();

        state.toggle_run("run-0");
        state.toggle_run("run-3");
        state.toggle_run("run-5");
        state.cache_metrics("run-0", vec![make_metric("loss", &[1.0])]);
        state.cache_metrics("run-3", vec![make_metric("loss", &[0.9])]);
        state.cache_metrics("run-5", vec![make_metric("loss", &[0.8])]);

        // Prune: only run-0 and run-3 are valid
        let valid_ids: HashSet<String> =
            ["run-0", "run-3"].iter().map(|s| s.to_string()).collect();
        state.prune_invalid_runs(&valid_ids);

        assert!(state.is_marked("run-0"));
        assert!(state.is_marked("run-3"));
        assert!(!state.is_marked("run-5"));
        assert!(state.get_cached_metrics("run-0").is_some());
        assert!(state.get_cached_metrics("run-3").is_some());
        assert!(state.get_cached_metrics("run-5").is_none());
    }

    #[test]
    fn test_toggle_removes_cached_metrics() {
        let mut state = ComparisonState::new();

        state.toggle_run("run-0");
        state.cache_metrics("run-0", vec![make_metric("loss", &[1.0])]);
        assert!(state.get_cached_metrics("run-0").is_some());

        // Toggle off should also remove cached metrics
        state.toggle_run("run-0");
        assert!(state.get_cached_metrics("run-0").is_none());
    }

    #[test]
    fn test_has_comparisons() {
        let mut state = ComparisonState::new();

        assert!(!state.has_comparisons());

        state.toggle_run("run-0");
        assert!(state.has_comparisons());

        state.toggle_run("run-0");
        assert!(!state.has_comparisons());
    }
}
