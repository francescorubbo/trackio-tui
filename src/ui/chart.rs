//! Metrics chart widget for visualizing training metrics.

use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    symbols::Marker,
    text::{Line, Span},
    widgets::{Axis, Block, Borders, Chart, Clear, Dataset, GraphType, LegendPosition},
    Frame,
};

use crate::data::Metric;
use super::theme::Theme;

#[cfg(test)]
use crate::data::MetricPoint;

/// Metrics chart widget for displaying line plots
pub struct MetricsChart<'a> {
    metrics: &'a [(String, usize, &'a Metric)], // (run_name, run_idx, metric)
    title: &'a str,
    theme: &'a Theme,
}

impl<'a> MetricsChart<'a> {
    pub fn new(
        metrics: &'a [(String, usize, &'a Metric)],
        title: &'a str,
        theme: &'a Theme,
    ) -> Self {
        MetricsChart { metrics, title, theme }
    }

    pub fn render(&self, frame: &mut Frame, area: Rect, focused: bool) {
        if self.metrics.is_empty() {
            self.render_empty(frame, area, focused);
            return;
        }

        // Step 1: Convert metric points directly to chart data format
        // Each entry is (run_idx, color, data_points)
        let chart_data: Vec<(usize, ratatui::style::Color, Vec<(f64, f64)>)> = self.metrics
            .iter()
            .map(|(_, run_idx, metric)| {
                let color = self.theme.chart_color(*run_idx);
                let points: Vec<(f64, f64)> = metric.points
                    .iter()
                    .map(|p| (p.step as f64, p.value))
                    .collect();
                (*run_idx, color, points)
            })
            .collect();

        // Step 2: Calculate bounds from all data
        let (x_bounds, y_bounds) = calculate_bounds(&chart_data);

        // Step 3: Create datasets - reference the data we just collected
        let datasets: Vec<Dataset> = self.metrics
            .iter()
            .zip(chart_data.iter())
            .map(|((run_name, _, _), (_, color, points))| {
                // Create colored label for legend (only show if multiple runs)
                let label: Line = if self.metrics.len() > 1 {
                    Line::from(Span::styled(run_name.clone(), Style::default().fg(*color)))
                } else {
                    Line::from("")
                };
                
                Dataset::default()
                    .name(label)
                    .marker(Marker::Braille)
                    .graph_type(GraphType::Line)
                    .style(Style::default().fg(*color))
                    .data(points)
            })
            .collect();

        // Step 4: Build and render the chart
        let (border_style, title_style) = self.theme.panel_styles(focused);
        let label_style = self.theme.surface_style();
        
        let x_labels = vec![
            Span::styled(format!("{:.0}", x_bounds.0), label_style),
            Span::styled(format!("{:.0}", (x_bounds.0 + x_bounds.1) / 2.0), label_style),
            Span::styled(format!("{:.0}", x_bounds.1), label_style),
        ];

        let y_labels = vec![
            Span::styled(format_value(y_bounds.0), label_style),
            Span::styled(format_value((y_bounds.0 + y_bounds.1) / 2.0), label_style),
            Span::styled(format_value(y_bounds.1), label_style),
        ];

        let chart = Chart::new(datasets)
            .block(
                Block::default()
                    .title(format!(" {} ", self.title))
                    .borders(Borders::ALL)
                    .border_style(border_style)
                    .title_style(title_style),
            )
            .x_axis(
                Axis::default()
                    .title(Span::styled("step", Style::default().add_modifier(Modifier::DIM)))
                    .style(label_style)
                    .bounds([x_bounds.0, x_bounds.1])
                    .labels(x_labels),
            )
            .y_axis(
                Axis::default()
                    .style(label_style)
                    .bounds([y_bounds.0, y_bounds.1])
                    .labels(y_labels),
            )
            // Explicitly set legend position - None when single run, TopRight for comparison
            .legend_position(if self.metrics.len() > 1 {
                Some(LegendPosition::TopRight)
            } else {
                None
            })
            // Set explicit style to prevent color bleeding
            .style(self.theme.surface_style());

        // Clear the area first to prevent style bleeding from previous frames
        frame.render_widget(Clear, area);
        frame.render_widget(chart, area);
    }

    fn render_empty(&self, frame: &mut Frame, area: Rect, focused: bool) {
        // Clear the area first to prevent style bleeding
        frame.render_widget(Clear, area);
        
        let (border_style, title_style) = self.theme.panel_styles(focused);

        let block = Block::default()
            .title(format!(" {} ", self.title))
            .borders(Borders::ALL)
            .border_style(border_style)
            .title_style(title_style)
            .style(self.theme.surface_style());

        let inner = block.inner(area);
        frame.render_widget(block, area);

        let message = ratatui::widgets::Paragraph::new("No data available")
            .style(self.theme.surface_style().add_modifier(Modifier::DIM))
            .alignment(ratatui::layout::Alignment::Center);

        frame.render_widget(message, inner);
    }
}

/// Calculate X and Y bounds from chart data with padding
fn calculate_bounds(
    data: &[(usize, ratatui::style::Color, Vec<(f64, f64)>)]
) -> ((f64, f64), (f64, f64)) {
    let mut x_min = f64::MAX;
    let mut x_max = f64::MIN;
    let mut y_min = f64::MAX;
    let mut y_max = f64::MIN;

    for (_, _, points) in data {
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

    // Add padding to y-axis
    let y_range = y_max - y_min;
    y_min -= y_range * 0.05;
    y_max += y_range * 0.05;

    ((x_min, x_max), (y_min, y_max))
}

/// Metric selector bar widget
pub struct MetricSelector<'a> {
    metrics: &'a [String],
    selected: usize,
    theme: &'a Theme,
}

impl<'a> MetricSelector<'a> {
    pub fn new(metrics: &'a [String], selected: usize, theme: &'a Theme) -> Self {
        MetricSelector {
            metrics,
            selected,
            theme,
        }
    }

    pub fn render(&self, frame: &mut Frame, area: Rect) {
        // Clear selector area to avoid ghosting when metric list shrinks
        frame.render_widget(Clear, area);

        let spans: Vec<Span> = self
            .metrics
            .iter()
            .enumerate()
            .flat_map(|(i, name)| {
                let num = format!("[{}] ", i + 1);
                let style = if i == self.selected {
                    self.theme.highlight_style()
                } else {
                    self.theme.normal_style()
                };
                vec![
                    Span::styled(num, Style::default().add_modifier(Modifier::DIM)),
                    Span::styled(format!("{name}  "), style),
                ]
            })
            .collect();

        let line = ratatui::text::Line::from(spans);
        let paragraph = ratatui::widgets::Paragraph::new(line)
            .style(self.theme.surface_style());

        frame.render_widget(paragraph, area);
    }
}

/// Format a value for display on axis labels
fn format_value(value: f64) -> String {
    let abs_val = value.abs();
    if (abs_val < 0.001 && value != 0.0) || abs_val >= 1000.0 {
        format!("{value:.2e}")
    } else if abs_val >= 1.0 {
        format!("{value:.2}")
    } else {
        format!("{value:.4}")
    }
}

/// Extract color assignments from metrics for testing
/// Returns Vec of (run_name, run_idx, assigned_color)
#[cfg(test)]
pub fn get_color_assignments(
    metrics: &[(String, usize, &Metric)],
    theme: &Theme,
) -> Vec<(String, usize, ratatui::style::Color)> {
    metrics
        .iter()
        .map(|(run_name, run_idx, _)| {
            (run_name.clone(), *run_idx, theme.chart_color(*run_idx))
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::buffer::Buffer;
    use ratatui::layout::Rect;
    use ratatui::style::Color;
    use ratatui::widgets::Widget;

    fn make_test_metric(name: &str, values: &[f64]) -> Metric {
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
    fn test_color_assignment_uses_run_index() {
        let theme = Theme::default();
        let metric = make_test_metric("loss", &[1.0, 0.5, 0.3]);

        // Simulate runs at indices 0, 3, 5 (not consecutive)
        let metrics: Vec<(String, usize, &Metric)> = vec![
            ("run_a".to_string(), 0, &metric),
            ("run_b".to_string(), 3, &metric),
            ("run_c".to_string(), 5, &metric),
        ];

        let assignments = get_color_assignments(&metrics, &theme);

        // Each run should get color based on its run_idx, not its position in the array
        assert_eq!(assignments[0].2, theme.chart_color(0), "run_a should get color 0");
        assert_eq!(assignments[1].2, theme.chart_color(3), "run_b should get color 3");
        assert_eq!(assignments[2].2, theme.chart_color(5), "run_c should get color 5");
    }

    #[test]
    fn test_color_assignment_consistency() {
        let theme = Theme::default();
        let metric = make_test_metric("loss", &[1.0, 0.5]);

        // Same run_idx should always give same color, regardless of position
        let metrics1: Vec<(String, usize, &Metric)> = vec![
            ("run_a".to_string(), 2, &metric),
        ];
        let metrics2: Vec<(String, usize, &Metric)> = vec![
            ("run_x".to_string(), 0, &metric),
            ("run_a".to_string(), 2, &metric), // same run_idx=2
        ];

        let assignments1 = get_color_assignments(&metrics1, &theme);
        let assignments2 = get_color_assignments(&metrics2, &theme);

        // Run with index 2 should have same color in both cases
        assert_eq!(assignments1[0].2, assignments2[1].2,
            "Same run_idx should yield same color regardless of array position");
    }

    #[test]
    fn test_colors_are_distinct_for_different_runs() {
        let theme = Theme::default();
        let metric = make_test_metric("loss", &[1.0]);

        let metrics: Vec<(String, usize, &Metric)> = vec![
            ("run_0".to_string(), 0, &metric),
            ("run_1".to_string(), 1, &metric),
            ("run_2".to_string(), 2, &metric),
        ];

        let assignments = get_color_assignments(&metrics, &theme);

        // All colors should be different
        assert_ne!(assignments[0].2, assignments[1].2);
        assert_ne!(assignments[1].2, assignments[2].2);
        assert_ne!(assignments[0].2, assignments[2].2);
    }

    #[test]
    fn test_chart_color_is_not_gray() {
        let theme = Theme::default();
        let metric = make_test_metric("loss", &[1.0]);

        let metrics: Vec<(String, usize, &Metric)> = vec![
            ("run_0".to_string(), 0, &metric),
        ];

        let assignments = get_color_assignments(&metrics, &theme);
        
        match assignments[0].2 {
            Color::Rgb(r, g, b) => {
                // Verify it's not grayscale
                let is_gray = r == g && g == b;
                assert!(!is_gray, "Chart color should not be gray: ({r}, {g}, {b})");
            }
            color => {
                // Named colors like Color::Gray should fail this test
                assert!(
                    !matches!(color, Color::Gray | Color::DarkGray | Color::White | Color::Black),
                    "Chart color should not be a gray shade: {color:?}"
                );
            }
        }
    }

    #[test]
    fn test_dataset_renders_with_color_in_buffer() {
        // Create a dataset with a known color and render it to a buffer
        // to verify the color is actually applied during rendering
        let expected_color = Color::Red; // Named color from theme
        
        let datasets = vec![
            Dataset::default()
                .name("test")
                .marker(Marker::Braille) // Use Braille marker like in production code
                .graph_type(GraphType::Line)
                .style(Style::default().fg(expected_color))
                .data(&[(0.0, 0.0), (10.0, 10.0), (20.0, 5.0)]),
        ];

        let chart = Chart::new(datasets)
            .x_axis(Axis::default().bounds([0.0, 25.0]))
            .y_axis(Axis::default().bounds([0.0, 15.0]));

        let area = Rect::new(0, 0, 60, 20);
        let mut buffer = Buffer::empty(area);
        chart.render(area, &mut buffer);

        // Check if any cell in the buffer has the expected color
        let has_expected_color = buffer.content.iter().any(|cell| {
            cell.fg == expected_color
        });

        assert!(
            has_expected_color,
            "Buffer should contain cells with the dataset color {:?}. \
             Found colors: {:?}",
            expected_color,
            buffer.content.iter()
                .filter(|c| c.fg != Color::Reset && c.fg != Color::White)
                .map(|c| c.fg)
                .collect::<std::collections::HashSet<_>>()
        );
    }

    #[test]
    fn test_legend_text_has_color() {
        // Verify that the legend text uses the dataset color
        let expected_color = Color::Red;
        
        let label = Line::from(Span::styled("test_run", Style::default().fg(expected_color)));
        
        let datasets = vec![
            Dataset::default()
                .name(label)
                .marker(Marker::Braille)
                .graph_type(GraphType::Line)
                .style(Style::default().fg(expected_color))
                .data(&[(0.0, 0.0), (10.0, 10.0)]),
        ];

        let chart = Chart::new(datasets)
            .legend_position(Some(LegendPosition::TopRight))
            .x_axis(Axis::default().bounds([0.0, 15.0]))
            .y_axis(Axis::default().bounds([0.0, 15.0]));

        let area = Rect::new(0, 0, 80, 20);
        let mut buffer = Buffer::empty(area);
        chart.render(area, &mut buffer);

        // The legend should contain the colored text
        let has_expected_color = buffer.content.iter().any(|cell| {
            cell.fg == expected_color
        });

        assert!(
            has_expected_color,
            "Legend text should be rendered with the dataset color {:?}",
            expected_color
        );
    }

    #[test]
    fn test_minimal_chart_with_different_colors() {
        // Minimal test: Create 3 separate charts with different colors
        // and verify each renders with the correct color
        let colors = [Color::Red, Color::Green, Color::Yellow];
        
        for (i, expected_color) in colors.iter().enumerate() {
            let datasets = vec![
                Dataset::default()
                    .name(format!("run_{}", i))
                    .marker(Marker::Braille)
                    .graph_type(GraphType::Line)
                    .style(Style::default().fg(*expected_color))
                    .data(&[(0.0, 0.0), (5.0, 5.0), (10.0, 2.0)]),
            ];

            let chart = Chart::new(datasets)
                .x_axis(Axis::default().bounds([0.0, 12.0]))
                .y_axis(Axis::default().bounds([0.0, 7.0]));

            let area = Rect::new(0, 0, 40, 15);
            let mut buffer = Buffer::empty(area);
            chart.render(area, &mut buffer);

            // Collect all non-default colors in the buffer
            let found_colors: std::collections::HashSet<_> = buffer.content
                .iter()
                .filter(|c| c.fg != Color::Reset && c.fg != Color::White)
                .map(|c| c.fg)
                .collect();

            assert!(
                found_colors.contains(expected_color),
                "Chart {} should render with color {:?}, but found: {:?}",
                i, expected_color, found_colors
            );
        }
    }
}

