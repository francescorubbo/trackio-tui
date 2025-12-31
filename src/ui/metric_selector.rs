//! Metric slot selection state and widget.
//!
//! Implements a slot-based model where up to 9 metrics are shown in slots,
//! and the user selects a slot to visualize. The window of metrics can be
//! shifted while keeping the slot selection fixed.

use std::collections::HashSet;

use ratatui::{layout::Rect, widgets::Paragraph, Frame};

/// Maximum number of slots displayed
pub const MAX_SLOTS: usize = 9;

/// State for metric slot selection.
///
/// The metric selector shows up to 9 "slots" containing metrics from a sliding window.
/// The user selects a slot (1-9), and the visualization shows whichever metric
/// currently occupies that slot. Multiple metrics can be selected for overlay.
#[derive(Debug, Clone, Default)]
pub struct MetricSlotState {
    /// Which slot (0-8) has the * marker (focus)
    pub selected_slot: usize,
    /// Which metric index is at slot 0
    pub window_start: usize,
    /// Names of metrics selected for overlay display
    pub selected_metrics: HashSet<String>,
}

impl MetricSlotState {
    /// Create a new MetricSlotState with default values
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns the index of the currently visualized metric.
    ///
    /// This is the metric at `(window_start + selected_slot) % num_metrics`.
    pub fn selected_metric(&self, num_metrics: usize) -> usize {
        if num_metrics == 0 {
            return 0;
        }
        (self.window_start + self.selected_slot) % num_metrics
    }

    /// Select a slot (0-indexed).
    ///
    /// The slot must be valid for the current number of metrics.
    /// Invalid slots are ignored.
    pub fn select_slot(&mut self, slot: usize, num_metrics: usize) {
        if num_metrics == 0 {
            return;
        }

        let num_visible = self.num_visible_slots(num_metrics);
        if slot < num_visible {
            self.selected_slot = slot;
        }
    }

    /// Shift the window left (show earlier metrics).
    ///
    /// Uses circular wrapping - metrics rotate smoothly.
    pub fn shift_left(&mut self, num_metrics: usize) {
        if num_metrics <= MAX_SLOTS {
            // No shifting needed when all metrics fit
            return;
        }

        // Circular decrement: (window_start - 1) mod num_metrics
        self.window_start = (self.window_start + num_metrics - 1) % num_metrics;
    }

    /// Shift the window right (show later metrics).
    ///
    /// Uses circular wrapping - metrics rotate smoothly.
    pub fn shift_right(&mut self, num_metrics: usize) {
        if num_metrics <= MAX_SLOTS {
            // No shifting needed when all metrics fit
            return;
        }

        // Circular increment: (window_start + 1) mod num_metrics
        self.window_start = (self.window_start + 1) % num_metrics;
    }

    /// Clamp state to valid range after metrics change.
    ///
    /// Call this after loading new metrics to ensure state is valid.
    pub fn clamp(&mut self, num_metrics: usize) {
        if num_metrics == 0 {
            self.selected_slot = 0;
            self.window_start = 0;
            self.selected_metrics.clear();
            return;
        }

        // Clamp window_start using modulo (any position is valid in circular model)
        self.window_start %= num_metrics;

        // Clamp selected_slot to visible range
        let num_visible = self.num_visible_slots(num_metrics);
        if self.selected_slot >= num_visible {
            self.selected_slot = num_visible.saturating_sub(1);
        }
    }

    /// Returns the number of visible slots (may be less than 9 if fewer metrics).
    pub fn num_visible_slots(&self, num_metrics: usize) -> usize {
        num_metrics.min(MAX_SLOTS)
    }

    /// Check if there are more metrics than visible slots.
    ///
    /// In circular model, if there are more metrics than slots,
    /// both directions have more (the window can rotate).
    pub fn has_more_left(&self, num_metrics: usize) -> bool {
        num_metrics > MAX_SLOTS
    }

    /// Check if there are more metrics than visible slots.
    ///
    /// In circular model, if there are more metrics than slots,
    /// both directions have more (the window can rotate).
    pub fn has_more_right(&self, num_metrics: usize) -> bool {
        num_metrics > MAX_SLOTS
    }

    /// Toggle a metric by name in/out of the overlay selection.
    ///
    /// If the metric is already selected, it is removed. Otherwise, it is added.
    pub fn toggle_metric(&mut self, metric_name: &str) {
        if self.selected_metrics.contains(metric_name) {
            self.selected_metrics.remove(metric_name);
        } else {
            self.selected_metrics.insert(metric_name.to_string());
        }
    }

    /// Clear all selected metrics from the overlay.
    pub fn clear_selection(&mut self) {
        self.selected_metrics.clear();
    }

    /// Get the set of selected metric names for overlay display.
    pub fn selected_metric_names(&self) -> &HashSet<String> {
        &self.selected_metrics
    }
}

/// Metric selector bar widget for displaying and selecting metrics
pub struct MetricSelector<'a> {
    metrics: &'a [String],
    state: &'a MetricSlotState,
}

impl<'a> MetricSelector<'a> {
    pub fn new(metrics: &'a [String], state: &'a MetricSlotState) -> Self {
        MetricSelector { metrics, state }
    }

    pub fn render(&self, frame: &mut Frame, area: Rect) {
        let num_metrics = self.metrics.len();
        let num_visible = self.state.num_visible_slots(num_metrics);

        let mut text = String::new();

        // Left indicator: show "<" if there are more metrics than slots (circular)
        if self.state.has_more_left(num_metrics) {
            text.push_str("< ");
        }

        // Render visible metrics (slots) using modular indexing
        for slot in 0..num_visible {
            let metric_idx = (self.state.window_start + slot) % num_metrics;
            let name = &self.metrics[metric_idx];
            let focus_marker = if slot == self.state.selected_slot {
                "*"
            } else {
                ""
            };
            let select_marker = if self.state.selected_metrics.contains(name) {
                "•"
            } else {
                ""
            };
            text.push_str(&format!(
                "[{}] {}{}{}  ",
                slot + 1,
                name,
                focus_marker,
                select_marker
            ));
        }

        // Right indicator: show ">" if there are more metrics than slots (circular)
        if self.state.has_more_right(num_metrics) {
            text.push_str(" >");
        }

        let paragraph = Paragraph::new(text);
        frame.render_widget(paragraph, area);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_default_state() {
        let state = MetricSlotState::new();
        assert_eq!(state.selected_slot, 0);
        assert_eq!(state.window_start, 0);
    }

    #[test]
    fn test_selected_metric_calculation() {
        let mut state = MetricSlotState::new();
        state.selected_slot = 2;
        state.window_start = 3;
        assert_eq!(state.selected_metric(12), 5); // (3 + 2) % 12 = 5

        // Test wrap-around
        state.window_start = 10;
        state.selected_slot = 4;
        assert_eq!(state.selected_metric(12), 2); // (10 + 4) % 12 = 2
    }

    #[test]
    fn test_select_slot_basic() {
        let mut state = MetricSlotState::new();
        state.select_slot(4, 12);
        assert_eq!(state.selected_slot, 4);
    }

    #[test]
    fn test_select_slot_out_of_range_ignored() {
        let mut state = MetricSlotState::new();
        state.selected_slot = 2;
        // Try to select slot 10 (out of range for 9 slots)
        state.select_slot(10, 12);
        assert_eq!(state.selected_slot, 2); // Unchanged

        // Try to select slot 5 when only 3 metrics exist
        state.select_slot(5, 3);
        assert_eq!(state.selected_slot, 2); // Unchanged
    }

    #[test]
    fn test_select_slot_zero_metrics() {
        let mut state = MetricSlotState::new();
        state.select_slot(0, 0);
        assert_eq!(state.selected_slot, 0);
    }

    #[test]
    fn test_shift_right_basic() {
        let mut state = MetricSlotState::new();
        state.shift_right(12);
        assert_eq!(state.window_start, 1);
        state.shift_right(12);
        assert_eq!(state.window_start, 2);
    }

    #[test]
    fn test_shift_left_basic() {
        let mut state = MetricSlotState::new();
        state.window_start = 3;
        state.shift_left(12);
        assert_eq!(state.window_start, 2);
        state.shift_left(12);
        assert_eq!(state.window_start, 1);
    }

    #[test]
    fn test_shift_right_circular() {
        let mut state = MetricSlotState::new();
        // Circular model: can start at any position
        state.window_start = 10;
        state.shift_right(12);
        assert_eq!(state.window_start, 11);
        state.shift_right(12);
        assert_eq!(state.window_start, 0); // Wrapped from 11 to 0
        state.shift_right(12);
        assert_eq!(state.window_start, 1);
    }

    #[test]
    fn test_shift_left_circular() {
        let mut state = MetricSlotState::new();
        state.window_start = 1;
        state.shift_left(12);
        assert_eq!(state.window_start, 0);
        state.shift_left(12);
        assert_eq!(state.window_start, 11); // Wrapped from 0 to 11
        state.shift_left(12);
        assert_eq!(state.window_start, 10);
    }

    #[test]
    fn test_fewer_than_9_metrics_no_shift() {
        let mut state = MetricSlotState::new();

        // With 5 metrics, shifting should do nothing
        state.shift_right(5);
        assert_eq!(state.window_start, 0);

        state.shift_left(5);
        assert_eq!(state.window_start, 0);
    }

    #[test]
    fn test_exactly_9_metrics_no_shift() {
        let mut state = MetricSlotState::new();

        state.shift_right(9);
        assert_eq!(state.window_start, 0);

        state.shift_left(9);
        assert_eq!(state.window_start, 0);
    }

    #[test]
    fn test_clamp_after_metrics_reduced() {
        let mut state = MetricSlotState::new();
        state.window_start = 5;
        state.selected_slot = 7;

        // Reduce to 6 metrics
        state.clamp(6);

        // window_start should be 5 % 6 = 5 (valid in circular model)
        assert_eq!(state.window_start, 5);
        // selected_slot should be clamped to 5 (max index for 6 slots)
        assert_eq!(state.selected_slot, 5);
    }

    #[test]
    fn test_clamp_wraps_window_start() {
        let mut state = MetricSlotState::new();
        state.window_start = 15;
        state.selected_slot = 2;

        // 6 metrics: window_start wraps via modulo
        state.clamp(6);

        assert_eq!(state.window_start, 3); // 15 % 6 = 3
        assert_eq!(state.selected_slot, 2); // Still valid
    }

    #[test]
    fn test_clamp_zero_metrics() {
        let mut state = MetricSlotState::new();
        state.window_start = 5;
        state.selected_slot = 3;

        state.clamp(0);

        assert_eq!(state.window_start, 0);
        assert_eq!(state.selected_slot, 0);
    }

    #[test]
    fn test_clamp_preserves_valid_state() {
        let mut state = MetricSlotState::new();
        state.window_start = 2;
        state.selected_slot = 3;

        // 12 metrics - state is already valid
        state.clamp(12);

        assert_eq!(state.window_start, 2);
        assert_eq!(state.selected_slot, 3);
    }

    #[test]
    fn test_num_visible_slots() {
        let state = MetricSlotState::new();

        assert_eq!(state.num_visible_slots(12), 9);
        assert_eq!(state.num_visible_slots(5), 5);
        assert_eq!(state.num_visible_slots(0), 0);
    }

    #[test]
    fn test_has_more_circular() {
        let state = MetricSlotState::new();

        // With more metrics than slots, both directions are available (circular)
        assert!(state.has_more_left(12));
        assert!(state.has_more_right(12));

        // With exactly 9 metrics, no shifting possible
        assert!(!state.has_more_left(9));
        assert!(!state.has_more_right(9));

        // With fewer than 9 metrics, no shifting possible
        assert!(!state.has_more_left(5));
        assert!(!state.has_more_right(5));
    }

    #[test]
    fn test_slot_selection_with_fewer_metrics() {
        let mut state = MetricSlotState::new();

        // With 3 metrics, can only select slots 0, 1, 2
        state.select_slot(2, 3);
        assert_eq!(state.selected_slot, 2);

        // Slot 3 is out of range
        state.select_slot(3, 3);
        assert_eq!(state.selected_slot, 2); // Unchanged
    }

    #[test]
    fn test_integration_scenario() {
        // Simulate user interaction: 12 metrics (a-l)
        let mut state = MetricSlotState::new();
        let num_metrics = 12;

        // Initial: slot 0 selected, showing metric 0 (a)
        assert_eq!(state.selected_metric(num_metrics), 0);

        // Press "3" - select slot 2
        state.select_slot(2, num_metrics);
        assert_eq!(state.selected_slot, 2);
        assert_eq!(state.selected_metric(num_metrics), 2); // metric c

        // Press "]" - shift window right
        state.shift_right(num_metrics);
        assert_eq!(state.window_start, 1);
        assert_eq!(state.selected_slot, 2); // Still slot 2
        assert_eq!(state.selected_metric(num_metrics), 3); // metric d (1 + 2)

        // Press "]" again
        state.shift_right(num_metrics);
        assert_eq!(state.window_start, 2);
        assert_eq!(state.selected_metric(num_metrics), 4); // metric e (2 + 2)

        // Press "[" - shift back
        state.shift_left(num_metrics);
        assert_eq!(state.window_start, 1);
        assert_eq!(state.selected_metric(num_metrics), 3); // metric d
    }

    #[test]
    fn test_circular_wrap_user_scenario() {
        // User scenario: 12 metrics, slot 5 selected, metric 12 in slot 5
        // Press "]" -> metric 1 should move into slot 5
        let mut state = MetricSlotState::new();
        let num_metrics = 12;

        // Let's set window_start = 7:
        // Slot 0 -> metric (7 + 0) % 12 = 7
        // Slot 1 -> metric (7 + 1) % 12 = 8
        // Slot 2 -> metric (7 + 2) % 12 = 9
        // Slot 3 -> metric (7 + 3) % 12 = 10
        // Slot 4 -> metric (7 + 4) % 12 = 11 (metric "12" if 1-indexed)
        // ...
        state.window_start = 7;
        state.selected_slot = 4;
        assert_eq!(state.selected_metric(num_metrics), 11); // "metric 12"

        // Press "]" - shift right by 1
        state.shift_right(num_metrics);
        assert_eq!(state.window_start, 8);
        assert_eq!(state.selected_slot, 4); // Still slot 4

        // Now slot 4 shows metric (8 + 4) % 12 = 0 (metric "1")
        assert_eq!(state.selected_metric(num_metrics), 0);
    }
}
