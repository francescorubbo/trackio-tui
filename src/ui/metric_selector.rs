//! Metric slot selection state management.
//!
//! Implements a slot-based model where up to 9 metrics are shown in slots,
//! and the user selects a slot to visualize. The window of metrics can be
//! shifted while keeping the slot selection fixed.

use std::ops::Range;

/// Maximum number of slots displayed
pub const MAX_SLOTS: usize = 9;

/// State for metric slot selection.
///
/// The metric selector shows up to 9 "slots" containing metrics from a sliding window.
/// The user selects a slot (1-9), and the visualization shows whichever metric
/// currently occupies that slot.
#[derive(Debug, Clone, Default)]
pub struct MetricSlotState {
    /// Which slot (0-8) has the * marker
    pub selected_slot: usize,
    /// Which metric index is at slot 0
    pub window_start: usize,
}

impl MetricSlotState {
    /// Create a new MetricSlotState with default values
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns the index of the currently visualized metric.
    ///
    /// This is the metric at `window_start + selected_slot`.
    pub fn selected_metric(&self) -> usize {
        self.window_start + self.selected_slot
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
    /// Wraps around to the end if at the beginning.
    pub fn shift_left(&mut self, num_metrics: usize) {
        if num_metrics <= MAX_SLOTS {
            // No shifting needed when all metrics fit
            return;
        }

        if self.window_start > 0 {
            self.window_start -= 1;
        } else {
            // Wrap to end
            self.window_start = num_metrics - MAX_SLOTS;
        }
    }

    /// Shift the window right (show later metrics).
    ///
    /// Wraps around to the beginning if at the end.
    pub fn shift_right(&mut self, num_metrics: usize) {
        if num_metrics <= MAX_SLOTS {
            // No shifting needed when all metrics fit
            return;
        }

        let max_start = num_metrics - MAX_SLOTS;
        if self.window_start < max_start {
            self.window_start += 1;
        } else {
            // Wrap to beginning
            self.window_start = 0;
        }
    }

    /// Clamp state to valid range after metrics change.
    ///
    /// Call this after loading new metrics to ensure state is valid.
    pub fn clamp(&mut self, num_metrics: usize) {
        if num_metrics == 0 {
            self.selected_slot = 0;
            self.window_start = 0;
            return;
        }

        // Clamp window_start
        let max_start = num_metrics.saturating_sub(MAX_SLOTS.min(num_metrics));
        self.window_start = self.window_start.min(max_start);

        // Clamp selected_slot to visible range
        let num_visible = self.num_visible_slots(num_metrics);
        if self.selected_slot >= num_visible {
            self.selected_slot = num_visible.saturating_sub(1);
        }
    }

    /// Calculate the range of metric indices currently visible.
    pub fn visible_range(&self, num_metrics: usize) -> Range<usize> {
        let end = (self.window_start + MAX_SLOTS).min(num_metrics);
        self.window_start..end
    }

    /// Returns the number of visible slots (may be less than 9 if fewer metrics).
    pub fn num_visible_slots(&self, num_metrics: usize) -> usize {
        let range = self.visible_range(num_metrics);
        range.end - range.start
    }

    /// Check if there are metrics before the visible window.
    pub fn has_more_left(&self) -> bool {
        self.window_start > 0
    }

    /// Check if there are metrics after the visible window.
    pub fn has_more_right(&self, num_metrics: usize) -> bool {
        self.window_start + MAX_SLOTS < num_metrics
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
        assert_eq!(state.selected_metric(), 5); // 3 + 2
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
    fn test_shift_right_wraps_to_start() {
        let mut state = MetricSlotState::new();
        state.window_start = 3; // max_start for 12 metrics is 12-9=3
        state.shift_right(12);
        assert_eq!(state.window_start, 0); // Wrapped
    }

    #[test]
    fn test_shift_left_wraps_to_end() {
        let mut state = MetricSlotState::new();
        state.window_start = 0;
        state.shift_left(12);
        assert_eq!(state.window_start, 3); // 12 - 9 = 3
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

        // window_start should be 0 (can't start at 5 with only 6 metrics)
        assert_eq!(state.window_start, 0);
        // selected_slot should be clamped to 5 (max index for 6 slots)
        assert_eq!(state.selected_slot, 5);
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
    fn test_visible_range() {
        let mut state = MetricSlotState::new();

        // Full window
        state.window_start = 0;
        assert_eq!(state.visible_range(12), 0..9);

        // Shifted window
        state.window_start = 3;
        assert_eq!(state.visible_range(12), 3..12);

        // Fewer than 9 metrics
        state.window_start = 0;
        assert_eq!(state.visible_range(5), 0..5);
    }

    #[test]
    fn test_num_visible_slots() {
        let state = MetricSlotState::new();

        assert_eq!(state.num_visible_slots(12), 9);
        assert_eq!(state.num_visible_slots(5), 5);
        assert_eq!(state.num_visible_slots(0), 0);
    }

    #[test]
    fn test_has_more_left() {
        let mut state = MetricSlotState::new();

        state.window_start = 0;
        assert!(!state.has_more_left());

        state.window_start = 1;
        assert!(state.has_more_left());
    }

    #[test]
    fn test_has_more_right() {
        let mut state = MetricSlotState::new();

        // 12 metrics, window at start - there's more to the right
        state.window_start = 0;
        assert!(state.has_more_right(12));

        // Window at end
        state.window_start = 3; // 12 - 9 = 3
        assert!(!state.has_more_right(12));

        // Fewer than 9 metrics - nothing more
        state.window_start = 0;
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

        // Initial: slot 0 selected, showing metric 0 (a)
        assert_eq!(state.selected_metric(), 0);

        // Press "3" - select slot 2
        state.select_slot(2, 12);
        assert_eq!(state.selected_slot, 2);
        assert_eq!(state.selected_metric(), 2); // metric c

        // Press "]" - shift window right
        state.shift_right(12);
        assert_eq!(state.window_start, 1);
        assert_eq!(state.selected_slot, 2); // Still slot 2
        assert_eq!(state.selected_metric(), 3); // metric d (1 + 2)

        // Press "]" again
        state.shift_right(12);
        assert_eq!(state.window_start, 2);
        assert_eq!(state.selected_metric(), 4); // metric e (2 + 2)

        // Press "[" - shift back
        state.shift_left(12);
        assert_eq!(state.window_start, 1);
        assert_eq!(state.selected_metric(), 3); // metric d
    }
}
