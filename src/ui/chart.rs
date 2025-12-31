//! Metrics chart widget for visualizing training metrics.

use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    symbols::Marker,
    text::{Line, Span},
    widgets::{Axis, Block, Borders, Chart, Dataset, GraphType, Paragraph},
    Frame,
};

use crate::data::Metric;

/// Colorblind-friendly palette (256-color approximation of Wong palette)
const COLORS: [Color; 6] = [
    Color::Indexed(32),  // Blue (#0087d7)
    Color::Indexed(214), // Orange (#ffaf00)
    Color::Indexed(36),  // Teal (#00af87)
    Color::Indexed(175), // Pink (#d787af)
    Color::Indexed(117), // Sky blue (#87d7ff)
    Color::Indexed(227), // Yellow (#ffff5f)
];

/// Markers for differentiating metrics
const MARKERS: [Marker; 4] = [
    Marker::Braille, 
    Marker::Dot, 
    Marker::Block,
    Marker::Quadrant,
];

/// Metric data selected for display.
/// Tuple: (run_name, run_idx, metric_idx, metric)
type RunMetric<'a> = (String, usize, usize, &'a Metric);

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
            .map(|(_, _, _, metric)| {
                metric
                    .points
                    .iter()
                    .map(|p| (p.step as f64, p.value))
                    .collect()
            })
            .collect();

        // Calculate bounds from all points
        let (x_bounds, y_bounds) = calculate_bounds(&chart_data);

        // Collect unique runs and metrics for factorized legend
        let mut unique_runs: Vec<(String, usize)> = Vec::new();
        let mut unique_metrics: Vec<(String, usize)> = Vec::new();
        for (run_name, run_idx, metric_idx, metric) in self.metrics.iter() {
            if !unique_runs.iter().any(|(_, idx)| idx == run_idx) {
                unique_runs.push((run_name.clone(), *run_idx));
            }
            if !unique_metrics.iter().any(|(_, idx)| idx == metric_idx) {
                unique_metrics.push((metric.name.clone(), *metric_idx));
            }
        }
        // Sort for consistent display
        unique_runs.sort_by_key(|(_, idx)| *idx);
        unique_metrics.sort_by_key(|(_, idx)| *idx);

        let multi_run = unique_runs.len() > 1;
        let multi_metric = unique_metrics.len() > 1;

        // Create datasets (no legend names - we use factorized legend)
        let datasets: Vec<Dataset> = self
            .metrics
            .iter()
            .zip(chart_data.iter())
            .map(|((_, run_idx, metric_idx, _), points)| {
                let color = COLORS[*run_idx % COLORS.len()];
                let marker = MARKERS[*metric_idx % MARKERS.len()];
                Dataset::default()
                    .name("")  // Empty name - we use custom legend
                    .marker(marker)
                    .graph_type(GraphType::Line)
                    .style(Style::default().fg(color))
                    .data(points)
            })
            .collect();

        // Split area: optional legend row + chart
        let show_legend = multi_run || multi_metric;
        let (legend_area, chart_area) = if show_legend {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Length(1), Constraint::Min(5)])
                .split(area);
            (Some(chunks[0]), chunks[1])
        } else {
            (None, area)
        };

        // Render factorized legend if needed (above chart)
        if let Some(legend_rect) = legend_area {
            let legend = self.build_factorized_legend(&unique_runs, &unique_metrics);
            frame.render_widget(legend, legend_rect);
        }

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
            .legend_position(None);  // Disabled - using custom factorized legend

        frame.render_widget(chart, chart_area);
    }

    /// Build a factorized legend showing runs by color and metrics by marker
    fn build_factorized_legend<'b>(
        &self,
        unique_runs: &[(String, usize)],
        unique_metrics: &[(String, usize)],
    ) -> Paragraph<'b> {
        let mut spans: Vec<Span> = Vec::new();

        // Add metrics with markers (if multiple)
        if unique_metrics.len() > 1 {
            for (i, (name, metric_idx)) in unique_metrics.iter().enumerate() {
                if i > 0 {
                    spans.push(Span::raw("  "));
                }
                let marker_char = match MARKERS[*metric_idx % MARKERS.len()] {
                    Marker::Braille => "⣿",
                    Marker::Dot => "•",
                    Marker::Block => "█",
                    _ => "·",
                };
                spans.push(Span::styled(
                    format!("{} ", marker_char),
                    Style::default().add_modifier(Modifier::DIM),
                ));
                spans.push(Span::raw(name.clone()));
            }
        }

        // Add separator if both
        if unique_metrics.len() > 1 && unique_runs.len() > 1 {
            spans.push(Span::raw("  │  "));
        }

        // Add runs with colors (if multiple)
        if unique_runs.len() > 1 {
            for (i, (name, run_idx)) in unique_runs.iter().enumerate() {
                if i > 0 {
                    spans.push(Span::raw("  "));
                }
                let color = COLORS[*run_idx % COLORS.len()];
                spans.push(Span::styled(
                    format!("■ {}", name),
                    Style::default().fg(color),
                ));
            }
        }

        Paragraph::new(Line::from(spans)).alignment(Alignment::Center)
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

/// Calculate X and Y bounds from chart data.
/// Returns default bounds (0,1) for each axis if data is empty.
fn calculate_bounds(data: &[Vec<(f64, f64)>]) -> ((f64, f64), (f64, f64)) {
    // Check if there's any data at all
    let has_data = data.iter().any(|points| !points.is_empty());
    if !has_data {
        return ((0.0, 1.0), (0.0, 1.0));
    }

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

    // Ensure valid bounds (for single-point case)
    if x_min >= x_max {
        x_max = x_min + 1.0;
    }
    if y_min >= y_max {
        y_max = y_min + 1.0;
    }

    ((x_min, x_max), (y_min, y_max))
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

    #[test]
    fn test_calculate_bounds_empty_data() {
        // Completely empty
        let data: Vec<Vec<(f64, f64)>> = vec![];
        let (x_bounds, y_bounds) = calculate_bounds(&data);
        assert_eq!(x_bounds, (0.0, 1.0));
        assert_eq!(y_bounds, (0.0, 1.0));

        // Empty inner vectors
        let data = vec![vec![], vec![]];
        let (x_bounds, y_bounds) = calculate_bounds(&data);
        assert_eq!(x_bounds, (0.0, 1.0));
        assert_eq!(y_bounds, (0.0, 1.0));
    }
}
