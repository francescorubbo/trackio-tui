//! UI widgets for the trackio dashboard.

use ratatui::{
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Wrap},
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

        let border_style = if focused {
            self.theme.title_style()
        } else {
            self.theme.border_style()
        };

        let list = List::new(items)
            .block(
                Block::default()
                    .title(" Projects ")
                    .borders(Borders::ALL)
                    .border_style(border_style)
                    .title_style(self.theme.title_style()),
            )
            .highlight_style(self.theme.highlight_style());

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
        let items: Vec<ListItem> = self
            .runs
            .iter()
            .enumerate()
            .map(|(i, run)| {
                let selected_marker = if i == self.selected { "▶" } else { " " };
                let comparison_marker = if self.selected_for_comparison.contains(&i) {
                    "●"
                } else {
                    " "
                };
                
                let status_display = run.status.display();
                let name = run.display_name();
                
                let status_style = self.theme.status_style(&run.status);
                
                let line = Line::from(vec![
                    Span::raw(format!("{selected_marker}{comparison_marker} ")),
                    Span::raw(format!("{name:<12} ")),
                    Span::styled(format!("[{status_display}]"), status_style),
                ]);

                let style = if i == self.selected {
                    self.theme.highlight_style()
                } else if self.selected_for_comparison.contains(&i) {
                    Style::default().add_modifier(Modifier::BOLD)
                } else {
                    self.theme.normal_style()
                };

                ListItem::new(line).style(style)
            })
            .collect();

        let title = format!(" Runs ({}) ", self.runs.len());
        let border_style = if focused {
            self.theme.title_style()
        } else {
            self.theme.border_style()
        };

        let list = List::new(items)
            .block(
                Block::default()
                    .title(title)
                    .borders(Borders::ALL)
                    .border_style(border_style)
                    .title_style(self.theme.title_style()),
            )
            .highlight_style(self.theme.highlight_style());

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

        let border_style = if focused {
            self.theme.title_style()
        } else {
            self.theme.border_style()
        };

        let paragraph = Paragraph::new(lines)
            .block(
                Block::default()
                    .title(" Config ")
                    .borders(Borders::ALL)
                    .border_style(border_style)
                    .title_style(self.theme.title_style()),
            )
            .wrap(Wrap { trim: true });

        frame.render_widget(paragraph, area);
    }
}

/// Status bar widget
pub struct StatusBar<'a> {
    project: Option<&'a str>,
    metric: Option<&'a str>,
    smoothing: f64,
    error: Option<&'a str>,
    theme: &'a Theme,
}

impl<'a> StatusBar<'a> {
    pub fn new(
        project: Option<&'a str>,
        metric: Option<&'a str>,
        smoothing: f64,
        error: Option<&'a str>,
        theme: &'a Theme,
    ) -> Self {
        StatusBar {
            project,
            metric,
            smoothing,
            error,
            theme,
        }
    }

    pub fn render(&self, frame: &mut Frame, area: Rect) {
        // If there's an error, show it prominently
        if let Some(error) = self.error {
            let content = Line::from(vec![
                Span::styled(" Error: ", Style::default().fg(self.theme.status_failed).add_modifier(Modifier::BOLD)),
                Span::styled(error, Style::default().fg(self.theme.status_failed)),
            ]);

            let paragraph = Paragraph::new(content)
                .style(self.theme.normal_style())
                .block(Block::default().borders(Borders::TOP).border_style(Style::default().fg(self.theme.status_failed)));

            frame.render_widget(paragraph, area);
            return;
        }

        let title = match (self.project, self.metric) {
            (Some(p), Some(m)) => format!(" trackio-tui: {p} | {m} "),
            (Some(p), None) => format!(" trackio-tui: {p} "),
            _ => " trackio-tui ".to_string(),
        };

        // Create smoothing bar visualization
        let smooth_percent = (self.smoothing * 5.0) as usize; // 0-20 range to 0-100%
        let bar_filled = smooth_percent / 5;
        let bar_empty = 20 - bar_filled;
        let smoothing_bar = format!(
            "Smoothing: [{}{}] {:.0}",
            "=".repeat(bar_filled),
            " ".repeat(bar_empty),
            self.smoothing * 20.0
        );

        let help_text = "[h] Help  [q] Quit";

        let content = Line::from(vec![
            Span::styled(&title, self.theme.title_style()),
            Span::raw("  "),
            Span::raw(&smoothing_bar),
            Span::raw("  "),
            Span::styled(help_text, Style::default().add_modifier(Modifier::DIM)),
        ]);

        let paragraph = Paragraph::new(content)
            .style(self.theme.normal_style())
            .block(Block::default().borders(Borders::TOP).border_style(self.theme.border_style()));

        frame.render_widget(paragraph, area);
    }
}

