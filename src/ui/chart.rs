//! Metrics chart widget for visualizing training metrics.

use ratatui::{
    layout::Rect,
    style::{Modifier, Style},
    symbols::Marker,
    text::Span,
    widgets::{Axis, Block, Borders, Chart, Dataset, GraphType},
    Frame,
};

use crate::data::{Metric, MetricPoint};
use super::theme::Theme;

/// Chart configuration
pub struct ChartConfig {
    pub x_min: Option<f64>,
    pub x_max: Option<f64>,
    pub smoothing: f64,
}

impl Default for ChartConfig {
    fn default() -> Self {
        ChartConfig {
            x_min: None,
            x_max: None,
            smoothing: 0.0,
        }
    }
}

/// Metrics chart widget for displaying line plots
pub struct MetricsChart<'a> {
    metrics: &'a [(String, &'a Metric)], // (run_name, metric)
    title: &'a str,
    config: &'a ChartConfig,
    theme: &'a Theme,
}

impl<'a> MetricsChart<'a> {
    pub fn new(
        metrics: &'a [(String, &'a Metric)],
        title: &'a str,
        config: &'a ChartConfig,
        theme: &'a Theme,
    ) -> Self {
        MetricsChart {
            metrics,
            title,
            config,
            theme,
        }
    }

    pub fn render(&self, frame: &mut Frame, area: Rect, focused: bool) {
        if self.metrics.is_empty() {
            self.render_empty(frame, area, focused);
            return;
        }

        // Prepare datasets
        let mut datasets: Vec<Dataset> = Vec::new();
        let mut all_points: Vec<Vec<(f64, f64)>> = Vec::new();

        // Calculate bounds
        let mut x_min = f64::MAX;
        let mut x_max = f64::MIN;
        let mut y_min = f64::MAX;
        let mut y_max = f64::MIN;

        for (i, (run_name, metric)) in self.metrics.iter().enumerate() {
            // Apply smoothing if needed
            let points: Vec<MetricPoint> = if self.config.smoothing > 0.0 {
                metric.smoothed(self.config.smoothing)
            } else {
                metric.points.clone()
            };

            // Convert to chart format
            let chart_points: Vec<(f64, f64)> = points
                .iter()
                .map(|p| (p.step as f64, p.value))
                .collect();

            // Update bounds
            for (x, y) in &chart_points {
                if *x < x_min {
                    x_min = *x;
                }
                if *x > x_max {
                    x_max = *x;
                }
                if *y < y_min {
                    y_min = *y;
                }
                if *y > y_max {
                    y_max = *y;
                }
            }

            all_points.push(chart_points);

            // Create dataset
            let color = self.theme.chart_color(i);
            let label = if self.metrics.len() > 1 {
                run_name.clone()
            } else {
                String::new()
            };

            datasets.push(
                Dataset::default()
                    .name(label)
                    .marker(Marker::Braille)
                    .graph_type(GraphType::Line)
                    .style(Style::default().fg(color))
                    .data(&[]) // Will be set after we have the data
            );
        }

        // Apply x-axis config overrides
        if let Some(min) = self.config.x_min {
            x_min = min;
        }
        if let Some(max) = self.config.x_max {
            x_max = max;
        }

        // Ensure we have valid bounds
        if x_min >= x_max {
            x_max = x_min + 1.0;
        }
        if y_min >= y_max {
            y_max = y_min + 1.0;
        }

        // Add some padding to y-axis
        let y_range = y_max - y_min;
        y_min -= y_range * 0.05;
        y_max += y_range * 0.05;

        // Recreate datasets with actual data references
        let datasets: Vec<Dataset> = self.metrics
            .iter()
            .enumerate()
            .zip(all_points.iter())
            .map(|((i, (run_name, _)), points)| {
                let color = self.theme.chart_color(i);
                let label = if self.metrics.len() > 1 {
                    run_name.clone()
                } else {
                    String::new()
                };
                
                Dataset::default()
                    .name(label)
                    .marker(Marker::Braille)
                    .graph_type(GraphType::Line)
                    .style(Style::default().fg(color))
                    .data(points)
            })
            .collect();

        // Format axis labels
        let x_labels = vec![
            Span::raw(format!("{:.0}", x_min)),
            Span::raw(format!("{:.0}", (x_min + x_max) / 2.0)),
            Span::raw(format!("{:.0}", x_max)),
        ];

        let y_labels = vec![
            Span::raw(format_value(y_min)),
            Span::raw(format_value((y_min + y_max) / 2.0)),
            Span::raw(format_value(y_max)),
        ];

        let border_style = if focused {
            self.theme.title_style()
        } else {
            self.theme.border_style()
        };

        let chart = Chart::new(datasets)
            .block(
                Block::default()
                    .title(format!(" {} ", self.title))
                    .borders(Borders::ALL)
                    .border_style(border_style)
                    .title_style(self.theme.title_style()),
            )
            .x_axis(
                Axis::default()
                    .title(Span::styled("step", Style::default().add_modifier(Modifier::DIM)))
                    .style(self.theme.normal_style())
                    .bounds([x_min, x_max])
                    .labels(x_labels),
            )
            .y_axis(
                Axis::default()
                    .style(self.theme.normal_style())
                    .bounds([y_min, y_max])
                    .labels(y_labels),
            );

        frame.render_widget(chart, area);
    }

    fn render_empty(&self, frame: &mut Frame, area: Rect, focused: bool) {
        let border_style = if focused {
            self.theme.title_style()
        } else {
            self.theme.border_style()
        };

        let block = Block::default()
            .title(format!(" {} ", self.title))
            .borders(Borders::ALL)
            .border_style(border_style)
            .title_style(self.theme.title_style());

        let inner = block.inner(area);
        frame.render_widget(block, area);

        let message = ratatui::widgets::Paragraph::new("No data available")
            .style(Style::default().add_modifier(Modifier::DIM))
            .alignment(ratatui::layout::Alignment::Center);

        frame.render_widget(message, inner);
    }
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
                    Span::styled(format!("{}  ", name), style),
                ]
            })
            .collect();

        let line = ratatui::text::Line::from(spans);
        let paragraph = ratatui::widgets::Paragraph::new(line)
            .style(self.theme.normal_style());

        frame.render_widget(paragraph, area);
    }
}

/// Format a value for display on axis labels
fn format_value(value: f64) -> String {
    if value.abs() < 0.001 && value != 0.0 {
        format!("{:.2e}", value)
    } else if value.abs() >= 1000.0 {
        format!("{:.2e}", value)
    } else if value.abs() >= 1.0 {
        format!("{:.2}", value)
    } else {
        format!("{:.4}", value)
    }
}

