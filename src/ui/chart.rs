//! Metrics chart widget for visualizing training metrics.

use ratatui::{
    layout::Rect,
    style::{Color, Style},
    symbols::Marker,
    widgets::{Axis, Block, Borders, Chart, Dataset, GraphType, LegendPosition},
    Frame,
};

use crate::data::Metric;

/// Metric data selected for display.
type RunMetric<'a> = (String, usize, &'a Metric);

/// Metrics chart widget for displaying line plots
pub struct MetricsChart<'a> {
    metrics: &'a [RunMetric<'a>],
    title: &'a str,
}

impl<'a> MetricsChart<'a> {
    pub fn new(metrics: &'a [RunMetric<'a>], title: &'a str) -> Self {
        MetricsChart { metrics, title }
    }

    pub fn render(&self, frame: &mut Frame, area: Rect) {
        if self.metrics.is_empty() {
            self.render_empty(frame, area);
            return;
        }

        // Build points for each run
        let chart_data: Vec<Vec<(f64, f64)>> = self
            .metrics
            .iter()
            .map(|(_, _, metric)| {
                metric
                    .points
                    .iter()
                    .map(|p| (p.step as f64, p.value))
                    .collect()
            })
            .collect();

        // Calculate bounds from all points
        let (x_bounds, y_bounds) = calculate_bounds(&chart_data);

        // Create datasets
        // Colorblind-friendly palette (256-color approximation of Wong palette)
        const COLORS: [Color; 6] = [
            Color::Indexed(32),  // Blue (#0087d7)
            Color::Indexed(214), // Orange (#ffaf00)
            Color::Indexed(36),  // Teal (#00af87)
            Color::Indexed(175), // Pink (#d787af)
            Color::Indexed(117), // Sky blue (#87d7ff)
            Color::Indexed(227), // Yellow (#ffff5f)
        ];
        const MARKERS: [Marker; 3] = [Marker::Braille, Marker::Dot, Marker::Block];
        let datasets: Vec<Dataset> = self
            .metrics
            .iter()
            .zip(chart_data.iter())
            .enumerate()
            .map(|(i, ((run_name, _, _), points))| {
                Dataset::default()
                    .name(run_name.clone())
                    .marker(MARKERS[(i / COLORS.len()) % MARKERS.len()])
                    .graph_type(GraphType::Line)
                    .style(Style::default().fg(COLORS[i % COLORS.len()]))
                    .data(points)
            })
            .collect();

        let chart = Chart::new(datasets)
            .block(
                Block::default()
                    .title(format!(" {} ", self.title))
                    .borders(Borders::ALL),
            )
            .x_axis(
                Axis::default()
                    .title("step")
                    .bounds([x_bounds.0, x_bounds.1])
                    .labels(vec![
                        format!("{:.0}", x_bounds.0),
                        format!("{:.0}", x_bounds.1),
                    ]),
            )
            .y_axis(
                Axis::default()
                    .bounds([y_bounds.0, y_bounds.1])
                    .labels(vec![
                        format!("{:.2}", y_bounds.0),
                        format!("{:.2}", y_bounds.1),
                    ]),
            )
            .legend_position(if self.metrics.len() > 1 {
                Some(LegendPosition::TopRight)
            } else {
                None
            });

        frame.render_widget(chart, area);
    }

    fn render_empty(&self, frame: &mut Frame, area: Rect) {
        let block = Block::default()
            .title(format!(" {} ", self.title))
            .borders(Borders::ALL);

        let inner = block.inner(area);
        frame.render_widget(block, area);

        let message = ratatui::widgets::Paragraph::new("No data available")
            .alignment(ratatui::layout::Alignment::Center);

        frame.render_widget(message, inner);
    }
}

/// Calculate X and Y bounds from chart data
fn calculate_bounds(data: &[Vec<(f64, f64)>]) -> ((f64, f64), (f64, f64)) {
    let mut x_min = f64::MAX;
    let mut x_max = f64::MIN;
    let mut y_min = f64::MAX;
    let mut y_max = f64::MIN;

    for points in data {
        for &(x, y) in points {
            x_min = x_min.min(x);
            x_max = x_max.max(x);
            y_min = y_min.min(y);
            y_max = y_max.max(y);
        }
    }

    // Ensure valid bounds
    if x_min >= x_max {
        x_max = x_min + 1.0;
    }
    if y_min >= y_max {
        y_max = y_min + 1.0;
    }

    ((x_min, x_max), (y_min, y_max))
}

use crate::ui::metric_selector::MetricSlotState;

/// Metric selector bar widget
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
        let range = self.state.visible_range(num_metrics);

        let mut text = String::new();

        // Left indicator: show "<" if there are metrics before the window
        if self.state.has_more_left() {
            text.push_str("< ");
        }

        // Render visible metrics (slots)
        for (slot, metric_idx) in range.clone().enumerate() {
            let name = &self.metrics[metric_idx];
            if slot == self.state.selected_slot {
                text.push_str(&format!("[{}] {}*  ", slot + 1, name));
            } else {
                text.push_str(&format!("[{}] {}  ", slot + 1, name));
            }
        }

        // Right indicator: show ">" if there are metrics after the window
        if self.state.has_more_right(num_metrics) {
            text.push_str(" >");
        }

        let paragraph = ratatui::widgets::Paragraph::new(text);
        frame.render_widget(paragraph, area);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::buffer::Buffer;
    use ratatui::layout::Rect;
    use ratatui::widgets::Widget;

    #[test]
    fn test_dataset_renders() {
        let datasets = vec![Dataset::default()
            .name("test")
            .marker(Marker::Braille)
            .graph_type(GraphType::Line)
            .data(&[(0.0, 0.0), (10.0, 10.0), (20.0, 5.0)])];

        let chart = Chart::new(datasets)
            .x_axis(Axis::default().bounds([0.0, 25.0]))
            .y_axis(Axis::default().bounds([0.0, 15.0]));

        let area = Rect::new(0, 0, 60, 20);
        let mut buffer = Buffer::empty(area);
        chart.render(area, &mut buffer);

        // Just verify something was rendered
        let non_empty = buffer.content.iter().any(|cell| cell.symbol() != " ");
        assert!(non_empty, "Chart should render something");
    }

    #[test]
    fn test_calculate_bounds() {
        let data = vec![vec![(0.0, 1.0), (5.0, 3.0)], vec![(2.0, 0.5), (10.0, 2.0)]];

        let (x_bounds, y_bounds) = calculate_bounds(&data);

        assert_eq!(x_bounds, (0.0, 10.0));
        assert_eq!(y_bounds, (0.5, 3.0));
    }

    #[test]
    fn test_calculate_bounds_single_point() {
        let data = vec![vec![(5.0, 5.0)]];

        let (x_bounds, y_bounds) = calculate_bounds(&data);

        // Should expand to valid range
        assert_eq!(x_bounds, (5.0, 6.0));
        assert_eq!(y_bounds, (5.0, 6.0));
    }
}
