//! Run comparison state management.
//!
//! Manages which runs are marked for comparison and caches their metrics data.

use std::collections::{HashMap, HashSet};

use crate::data::Metric;

/// Manages run comparison state and cached metrics
#[derive(Debug, Default)]
pub struct ComparisonState {
    /// Run indices marked for comparison (HashSet for O(1) lookup)
    marked_runs: HashSet<usize>,
    /// Cached metrics for comparison, keyed by run index
    metrics_cache: HashMap<usize, Vec<Metric>>,
}

impl ComparisonState {
    /// Create a new empty comparison state
    pub fn new() -> Self {
        Self::default()
    }

    /// Toggle a run's comparison status. Returns true if run is now marked.
    pub fn toggle_run(&mut self, run_idx: usize) -> bool {
        if self.marked_runs.contains(&run_idx) {
            self.marked_runs.remove(&run_idx);
            self.metrics_cache.remove(&run_idx);
            false
        } else {
            self.marked_runs.insert(run_idx);
            true
        }
    }

    /// Check if a run is marked for comparison
    #[allow(dead_code)] // Used in tests
    pub fn is_marked(&self, run_idx: usize) -> bool {
        self.marked_runs.contains(&run_idx)
    }

    /// Get the set of marked run indices
    pub fn marked_runs(&self) -> &HashSet<usize> {
        &self.marked_runs
    }

    /// Clear all comparison state
    pub fn clear(&mut self) {
        self.marked_runs.clear();
        self.metrics_cache.clear();
    }

    /// Cache metrics for a run
    pub fn cache_metrics(&mut self, run_idx: usize, metrics: Vec<Metric>) {
        self.metrics_cache.insert(run_idx, metrics);
    }

    /// Get cached metrics for a specific run
    #[allow(dead_code)] // Used in tests
    pub fn get_cached_metrics(&self, run_idx: usize) -> Option<&Vec<Metric>> {
        self.metrics_cache.get(&run_idx)
    }

    /// Get comparison metrics for display, excluding the currently selected run.
    /// Returns an iterator of (run_idx, metric) pairs.
    pub fn get_comparison_metrics(
        &self,
        selected_run: usize,
    ) -> impl Iterator<Item = (usize, &Metric)> {
        self.marked_runs
            .iter()
            .filter(move |&&run_idx| run_idx != selected_run)
            .flat_map(|&run_idx| {
                self.metrics_cache
                    .get(&run_idx)
                    .into_iter()
                    .flat_map(move |metrics| metrics.iter().map(move |m| (run_idx, m)))
            })
    }

    /// Check if there are any runs marked for comparison
    #[allow(dead_code)] // Used in tests
    pub fn has_comparisons(&self) -> bool {
        !self.marked_runs.is_empty()
    }

    /// Remove runs that are no longer valid (index out of bounds)
    pub fn prune_invalid_runs(&mut self, max_run_idx: usize) {
        self.marked_runs.retain(|&run_idx| run_idx < max_run_idx);
        self.metrics_cache
            .retain(|&run_idx, _| run_idx < max_run_idx);
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
        assert!(!state.is_marked(0));
        assert!(state.marked_runs().is_empty());

        // Toggle on
        let added = state.toggle_run(0);
        assert!(added);
        assert!(state.is_marked(0));
        assert!(state.marked_runs().contains(&0));
        assert_eq!(state.marked_runs().len(), 1);

        // Toggle off
        let added = state.toggle_run(0);
        assert!(!added);
        assert!(!state.is_marked(0));
        assert!(state.marked_runs().is_empty());
    }

    #[test]
    fn test_multiple_runs_marked() {
        let mut state = ComparisonState::new();

        state.toggle_run(1);
        state.toggle_run(3);
        state.toggle_run(5);

        assert!(!state.is_marked(0));
        assert!(state.is_marked(1));
        assert!(!state.is_marked(2));
        assert!(state.is_marked(3));
        assert!(!state.is_marked(4));
        assert!(state.is_marked(5));
        assert_eq!(state.marked_runs().len(), 3);
        assert!(state.marked_runs().contains(&1));
        assert!(state.marked_runs().contains(&3));
        assert!(state.marked_runs().contains(&5));
    }

    #[test]
    fn test_clear_removes_all() {
        let mut state = ComparisonState::new();

        state.toggle_run(0);
        state.toggle_run(1);
        state.cache_metrics(0, vec![make_metric("loss", &[1.0, 0.5])]);
        state.cache_metrics(1, vec![make_metric("loss", &[0.9, 0.4])]);

        assert!(state.has_comparisons());
        assert!(state.get_cached_metrics(0).is_some());

        state.clear();

        assert!(!state.has_comparisons());
        assert!(state.marked_runs().is_empty());
        assert!(state.get_cached_metrics(0).is_none());
        assert!(state.get_cached_metrics(1).is_none());
    }

    #[test]
    fn test_get_comparison_metrics_excludes_selected() {
        let mut state = ComparisonState::new();

        // Mark runs 0, 1, 2
        state.toggle_run(0);
        state.toggle_run(1);
        state.toggle_run(2);

        state.cache_metrics(0, vec![make_metric("loss", &[1.0])]);
        state.cache_metrics(1, vec![make_metric("loss", &[0.9])]);
        state.cache_metrics(2, vec![make_metric("loss", &[0.8])]);

        // When selected_run is 1, we should get metrics for runs 0 and 2 only
        let comparison: Vec<(usize, &Metric)> = state.get_comparison_metrics(1).collect();

        assert_eq!(comparison.len(), 2);
        assert!(comparison.iter().any(|(idx, _)| *idx == 0));
        assert!(comparison.iter().any(|(idx, _)| *idx == 2));
        assert!(!comparison.iter().any(|(idx, _)| *idx == 1));
    }

    #[test]
    fn test_get_comparison_metrics_with_multiple_metrics_per_run() {
        let mut state = ComparisonState::new();

        state.toggle_run(0);
        state.cache_metrics(
            0,
            vec![
                make_metric("loss", &[1.0, 0.5]),
                make_metric("accuracy", &[0.5, 0.8]),
            ],
        );

        // When selected is 1, we get all metrics from run 0
        let comparison: Vec<(usize, &Metric)> = state.get_comparison_metrics(1).collect();

        assert_eq!(comparison.len(), 2);
        assert!(comparison.iter().all(|(idx, _)| *idx == 0));

        let names: Vec<&str> = comparison.iter().map(|(_, m)| m.name.as_str()).collect();
        assert!(names.contains(&"loss"));
        assert!(names.contains(&"accuracy"));
    }

    #[test]
    fn test_prune_invalid_runs() {
        let mut state = ComparisonState::new();

        state.toggle_run(0);
        state.toggle_run(3);
        state.toggle_run(5);
        state.cache_metrics(0, vec![make_metric("loss", &[1.0])]);
        state.cache_metrics(3, vec![make_metric("loss", &[0.9])]);
        state.cache_metrics(5, vec![make_metric("loss", &[0.8])]);

        // Prune to max index 4 (runs 0, 1, 2, 3 are valid; 5 is invalid)
        state.prune_invalid_runs(4);

        assert!(state.is_marked(0));
        assert!(state.is_marked(3));
        assert!(!state.is_marked(5));
        assert!(state.get_cached_metrics(0).is_some());
        assert!(state.get_cached_metrics(3).is_some());
        assert!(state.get_cached_metrics(5).is_none());
    }

    #[test]
    fn test_toggle_removes_cached_metrics() {
        let mut state = ComparisonState::new();

        state.toggle_run(0);
        state.cache_metrics(0, vec![make_metric("loss", &[1.0])]);
        assert!(state.get_cached_metrics(0).is_some());

        // Toggle off should also remove cached metrics
        state.toggle_run(0);
        assert!(state.get_cached_metrics(0).is_none());
    }

    #[test]
    fn test_has_comparisons() {
        let mut state = ComparisonState::new();

        assert!(!state.has_comparisons());

        state.toggle_run(0);
        assert!(state.has_comparisons());

        state.toggle_run(0);
        assert!(!state.has_comparisons());
    }
}
