//! UI widgets for the trackio dashboard.

use std::collections::HashSet;

use ratatui::{
    layout::Rect,
    style::{Color, Style},
    widgets::{Block, BorderType, Borders, List, ListItem, ListState, Paragraph},
    Frame,
};

use crate::data::{Config, Project, Run};

/// Project list panel widget
pub struct ProjectList<'a> {
    projects: &'a [Project],
    selected: usize,
}

impl<'a> ProjectList<'a> {
    pub fn new(projects: &'a [Project], selected: usize) -> Self {
        ProjectList { projects, selected }
    }

    pub fn render(&self, frame: &mut Frame, area: Rect, focused: bool) {
        let items: Vec<ListItem> = self
            .projects
            .iter()
            .map(|p| ListItem::new(format!("{} ({})", p.name, p.run_count)))
            .collect();

        let block = Block::default()
            .title(" Projects ")
            .borders(Borders::ALL)
            .border_type(if focused {
                BorderType::Double
            } else {
                BorderType::Plain
            })
            .border_style(if focused {
                Style::default().fg(Color::Cyan)
            } else {
                Style::default()
            });

        let list = List::new(items).block(block).highlight_symbol("> ");

        let mut state = ListState::default();
        state.select(Some(self.selected));
        frame.render_stateful_widget(list, area, &mut state);
    }
}

/// Run list panel widget
pub struct RunList<'a> {
    runs: &'a [Run],
    selected: usize,
    marked: &'a HashSet<usize>,
}

impl<'a> RunList<'a> {
    pub fn new(runs: &'a [Run], selected: usize, marked: &'a HashSet<usize>) -> Self {
        RunList {
            runs,
            selected,
            marked,
        }
    }

    pub fn render(&self, frame: &mut Frame, area: Rect, focused: bool) {
        let items: Vec<ListItem> = self
            .runs
            .iter()
            .enumerate()
            .map(|(idx, r)| {
                let prefix = if self.marked.contains(&idx) {
                    "● "
                } else {
                    "  "
                };
                ListItem::new(format!("{}{}", prefix, r.display_name))
            })
            .collect();

        let block = Block::default()
            .title(format!(" Runs ({}) ", self.runs.len()))
            .borders(Borders::ALL)
            .border_type(if focused {
                BorderType::Double
            } else {
                BorderType::Plain
            })
            .border_style(if focused {
                Style::default().fg(Color::Cyan)
            } else {
                Style::default()
            });

        let list = List::new(items).block(block).highlight_symbol("> ");

        let mut state = ListState::default();
        state.select(Some(self.selected));
        frame.render_stateful_widget(list, area, &mut state);
    }
}

/// Config panel state for scrolling and search
pub struct ConfigPanelState {
    pub scroll_v: u16,
    pub scroll_h: u16,
    pub search: String,
    pub search_active: bool,
    pub match_indices: Vec<usize>,
    pub current_match: usize,
}

/// Config panel widget
pub struct ConfigPanel<'a> {
    config: &'a [Config],
    state: &'a ConfigPanelState,
}

impl<'a> ConfigPanel<'a> {
    pub fn new(config: &'a [Config], state: &'a ConfigPanelState) -> Self {
        ConfigPanel { config, state }
    }

    pub fn render(&self, frame: &mut Frame, area: Rect, focused: bool) {
        use ratatui::text::{Line, Span};

        let lines: Vec<String> = self
            .config
            .iter()
            .map(|c| format!("{}: {}", c.key, c.value))
            .collect();

        // Build styled lines with search highlighting
        let styled_lines: Vec<Line> = lines
            .iter()
            .enumerate()
            .map(|(idx, line)| {
                let is_match_line = self.state.match_indices.contains(&idx);
                let is_current_match = self
                    .state
                    .match_indices
                    .get(self.state.current_match)
                    .map_or(false, |&m| m == idx);

                if !self.state.search.is_empty() && is_match_line {
                    // Highlight matching text
                    let query = &self.state.search.to_lowercase();
                    let line_lower = line.to_lowercase();
                    let mut spans = Vec::new();
                    let mut last_end = 0;

                    for (start, _) in line_lower.match_indices(query) {
                        if start > last_end {
                            spans.push(Span::raw(&line[last_end..start]));
                        }
                        let end = start + self.state.search.len();
                        let style = if is_current_match {
                            Style::default().fg(Color::Black).bg(Color::Yellow)
                        } else {
                            Style::default().fg(Color::Black).bg(Color::DarkGray)
                        };
                        spans.push(Span::styled(&line[start..end], style));
                        last_end = end;
                    }
                    if last_end < line.len() {
                        spans.push(Span::raw(&line[last_end..]));
                    }
                    Line::from(spans)
                } else {
                    Line::from(line.as_str())
                }
            })
            .collect();

        // Build title with match info
        let title = if self.state.search_active {
            if self.state.match_indices.is_empty() && !self.state.search.is_empty() {
                format!(" Config [/{}] (no matches) ", self.state.search)
            } else if !self.state.match_indices.is_empty() {
                format!(
                    " Config [/{}] ({}/{}) ",
                    self.state.search,
                    self.state.current_match + 1,
                    self.state.match_indices.len()
                )
            } else {
                format!(" Config [/{}] ", self.state.search)
            }
        } else if !self.state.search.is_empty() {
            if self.state.match_indices.is_empty() {
                format!(" Config [{}] (no matches) ", self.state.search)
            } else {
                format!(
                    " Config [{}] ({}/{}) ",
                    self.state.search,
                    self.state.current_match + 1,
                    self.state.match_indices.len()
                )
            }
        } else {
            " Config ".to_string()
        };

        let block = Block::default()
            .title(title)
            .borders(Borders::ALL)
            .border_type(if focused {
                BorderType::Double
            } else {
                BorderType::Plain
            })
            .border_style(if focused {
                Style::default().fg(Color::Cyan)
            } else {
                Style::default()
            });

        let paragraph = Paragraph::new(styled_lines)
            .block(block)
            .scroll((self.state.scroll_v, self.state.scroll_h));

        frame.render_widget(paragraph, area);
    }
}

/// Status bar widget
pub struct StatusBar<'a> {
    project: Option<&'a str>,
    error: Option<&'a str>,
}

impl<'a> StatusBar<'a> {
    pub fn new(project: Option<&'a str>, error: Option<&'a str>) -> Self {
        StatusBar { project, error }
    }

    pub fn render(&self, frame: &mut Frame, area: Rect) {
        let text = if let Some(e) = self.error {
            format!("Error: {e}")
        } else {
            match self.project {
                Some(p) => format!("trackio-tui: {p} | [h] Help [q] Quit"),
                None => "trackio-tui | [h] Help [q] Quit".to_string(),
            }
        };

        let paragraph = Paragraph::new(text).block(Block::default().borders(Borders::TOP));

        frame.render_widget(paragraph, area);
    }
}
