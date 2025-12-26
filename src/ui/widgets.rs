//! UI widgets for the trackio dashboard.

use ratatui::{
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph, Wrap},
    Frame,
};

use crate::data::{Config, Project, Run};
use super::theme::Theme;

/// Project list panel widget
pub struct ProjectList<'a> {
    projects: &'a [Project],
    selected: usize,
    theme: &'a Theme,
}

impl<'a> ProjectList<'a> {
    pub fn new(projects: &'a [Project], selected: usize, theme: &'a Theme) -> Self {
        ProjectList {
            projects,
            selected,
            theme,
        }
    }

    pub fn render(&self, frame: &mut Frame, area: Rect, focused: bool) {
        // Clear previous content to avoid style bleed when the panel shrinks
        frame.render_widget(Clear, area);

        let items: Vec<ListItem> = self
            .projects
            .iter()
            .enumerate()
            .map(|(i, project)| {
                let prefix = if i == self.selected { "▶ " } else { "  " };
                let content = format!("{}{} ({})", prefix, project.name, project.run_count);
                
                let style = if i == self.selected {
                    self.theme.highlight_style()
                } else {
                    self.theme.normal_style()
                };
                
                ListItem::new(content).style(style)
            })
            .collect();

        let (border_style, title_style) = self.theme.panel_styles(focused);

        let list = List::new(items)
            .block(
                Block::default()
                    .style(self.theme.surface_style())
                    .title(" Projects ")
                    .borders(Borders::ALL)
                    .border_style(border_style)
                    .title_style(title_style),
            )
            .highlight_style(self.theme.highlight_style())
            .style(self.theme.surface_style());

        let mut state = ListState::default();
        state.select(Some(self.selected));

        frame.render_stateful_widget(list, area, &mut state);
    }
}

/// Run list panel widget
pub struct RunList<'a> {
    runs: &'a [Run],
    selected: usize,
    selected_for_comparison: &'a [usize],
    theme: &'a Theme,
}

impl<'a> RunList<'a> {
    pub fn new(
        runs: &'a [Run],
        selected: usize,
        selected_for_comparison: &'a [usize],
        theme: &'a Theme,
    ) -> Self {
        RunList {
            runs,
            selected,
            selected_for_comparison,
            theme,
        }
    }

    pub fn render(&self, frame: &mut Frame, area: Rect, focused: bool) {
        // Clear previous content to avoid stale cells when list shrinks
        frame.render_widget(Clear, area);

        let items: Vec<ListItem> = self
            .runs
            .iter()
            .enumerate()
            .map(|(i, run)| {
                let selected_marker = if i == self.selected { "▶" } else { " " };
                
                // Show colored bullet for runs in chart (selected or comparison)
                let in_chart = i == self.selected || self.selected_for_comparison.contains(&i);
                let chart_color = self.theme.chart_color(i);
                
                let status_display = run.status.display();
                let name = run.display_name();
                
                let status_style = self.theme.status_style(&run.status);
                
                // Build the line with appropriate colors
                // For items in chart, use chart colors for name; otherwise use normal style
                let name_style = if in_chart {
                    Style::default().fg(chart_color)
                } else if i == self.selected {
                    self.theme.highlight_style()
                } else {
                    self.theme.normal_style()
                };

                let bullet_span = if in_chart {
                    Span::styled("●", Style::default().fg(chart_color))
                } else {
                    Span::raw(" ")
                };

                let line = Line::from(vec![
                    Span::raw(selected_marker),
                    bullet_span,
                    Span::raw(" "),
                    Span::styled(format!("{name:<12} "), name_style),
                    Span::styled(format!("[{status_display}]"), status_style),
                ]);

                // Only apply background highlight for selected item, don't override fg colors
                let item_style = if i == self.selected {
                    Style::default()
                        .bg(self.theme.highlight_bg)
                        .add_modifier(Modifier::BOLD)
                } else if self.selected_for_comparison.contains(&i) {
                    Style::default().add_modifier(Modifier::BOLD)
                } else {
                    Style::default()
                };

                ListItem::new(line).style(item_style)
            })
            .collect();

        let title = format!(" Runs ({}) ", self.runs.len());
        let (border_style, title_style) = self.theme.panel_styles(focused);

        let list = List::new(items)
            .block(
                Block::default()
                    .style(self.theme.surface_style())
                    .title(title)
                    .borders(Borders::ALL)
                    .border_style(border_style)
                    .title_style(title_style),
            )
            .highlight_style(self.theme.highlight_style())
            .style(self.theme.surface_style());

        let mut state = ListState::default();
        state.select(Some(self.selected));

        frame.render_stateful_widget(list, area, &mut state);
    }
}

/// Config panel widget
pub struct ConfigPanel<'a> {
    config: &'a [Config],
    theme: &'a Theme,
}

impl<'a> ConfigPanel<'a> {
    pub fn new(config: &'a [Config], theme: &'a Theme) -> Self {
        ConfigPanel { config, theme }
    }

    pub fn render(&self, frame: &mut Frame, area: Rect, focused: bool) {
        // Clear surface to avoid leftover text when content shrinks
        frame.render_widget(Clear, area);

        let lines: Vec<Line> = self
            .config
            .iter()
            .map(|c| {
                Line::from(vec![
                    Span::styled(
                        format!("{}: ", c.key),
                        Style::default().add_modifier(Modifier::BOLD),
                    ),
                    Span::raw(c.value.to_string()),
                ])
            })
            .collect();

        let (border_style, title_style) = self.theme.panel_styles(focused);

        let paragraph = Paragraph::new(lines)
            .block(
                Block::default()
                    .style(self.theme.surface_style())
                    .title(" Config ")
                    .borders(Borders::ALL)
                    .border_style(border_style)
                    .title_style(title_style),
            )
            .wrap(Wrap { trim: true })
            .style(self.theme.surface_style());

        frame.render_widget(paragraph, area);
    }
}

/// Status bar widget
pub struct StatusBar<'a> {
    project: Option<&'a str>,
    metric: Option<&'a str>,
    error: Option<&'a str>,
    theme: &'a Theme,
}

impl<'a> StatusBar<'a> {
    pub fn new(
        project: Option<&'a str>,
        metric: Option<&'a str>,
        error: Option<&'a str>,
        theme: &'a Theme,
    ) -> Self {
        StatusBar {
            project,
            metric,
            error,
            theme,
        }
    }

    pub fn render(&self, frame: &mut Frame, area: Rect) {
        // Clear status bar area to avoid mixing styles with previous frames
        frame.render_widget(Clear, area);

        // If there's an error, show it prominently
        if let Some(error) = self.error {
            let content = Line::from(vec![
                Span::styled(" Error: ", Style::default().fg(self.theme.status_failed).add_modifier(Modifier::BOLD)),
                Span::styled(error, Style::default().fg(self.theme.status_failed)),
            ]);

            let paragraph = Paragraph::new(content)
                .style(self.theme.surface_style())
                .block(
                    Block::default()
                        .style(self.theme.surface_style())
                        .borders(Borders::TOP)
                        .border_style(Style::default().fg(self.theme.status_failed)),
                );

            frame.render_widget(paragraph, area);
            return;
        }

        let title = match (self.project, self.metric) {
            (Some(p), Some(m)) => format!(" trackio-tui: {p} | {m} "),
            (Some(p), None) => format!(" trackio-tui: {p} "),
            _ => " trackio-tui ".to_string(),
        };

        let help_text = "[h] Help  [q] Quit";

        let content = Line::from(vec![
            Span::styled(&title, self.theme.title_style()),
            Span::raw("  "),
            Span::styled(help_text, Style::default().add_modifier(Modifier::DIM)),
        ]);

        let paragraph = Paragraph::new(content)
            .style(self.theme.surface_style())
            .block(
                Block::default()
                    .style(self.theme.surface_style())
                    .borders(Borders::TOP)
                    .border_style(self.theme.border_style()),
            );

        frame.render_widget(paragraph, area);
    }
}

