//! UI widgets for the trackio dashboard.

use std::collections::HashSet;

use ratatui::{
    layout::Rect,
    style::{Color, Style},
    widgets::{Block, BorderType, Borders, List, ListItem, ListState, Paragraph},
    Frame,
};

use crate::data::{Config, Project, Run};

/// Find case-insensitive matches, returning byte ranges in the original string.
/// Handles Unicode correctly by mapping lowercase byte positions back to original positions.
fn case_insensitive_byte_ranges(line: &str, query: &str) -> Vec<(usize, usize)> {
    if query.is_empty() {
        return Vec::new();
    }

    let line_lower = line.to_lowercase();
    let query_lower = query.to_lowercase();

    // Fast path: if byte lengths are equal, positions are compatible
    if line.len() == line_lower.len() {
        return line_lower
            .match_indices(&query_lower)
            .map(|(start, s)| (start, start + s.len()))
            .collect();
    }

    // Slow path: build byte position mapping from lowercase to original
    let mut lower_to_orig: Vec<usize> = Vec::with_capacity(line_lower.len() + 1);
    let mut orig_idx = 0;

    for c in line.chars() {
        let orig_len = c.len_utf8();
        let lower_str: String = c.to_lowercase().collect();
        for _ in 0..lower_str.len() {
            lower_to_orig.push(orig_idx);
        }
        orig_idx += orig_len;
    }
    lower_to_orig.push(orig_idx); // sentinel for end position

    line_lower
        .match_indices(&query_lower)
        .filter_map(|(start, s)| {
            let end = start + s.len();
            // Ensure indices are in bounds
            if end <= lower_to_orig.len() {
                Some((
                    lower_to_orig[start],
                    lower_to_orig[end.min(lower_to_orig.len() - 1)],
                ))
            } else {
                None
            }
        })
        .collect()
}

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
    marked_ids: &'a HashSet<String>,
}

impl<'a> RunList<'a> {
    pub fn new(runs: &'a [Run], selected: usize, marked_ids: &'a HashSet<String>) -> Self {
        RunList {
            runs,
            selected,
            marked_ids,
        }
    }

    pub fn render(&self, frame: &mut Frame, area: Rect, focused: bool) {
        let items: Vec<ListItem> = self
            .runs
            .iter()
            .map(|r| {
                let prefix = if self.marked_ids.contains(&r.id) {
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
#[derive(Debug, Default)]
pub struct ConfigPanelState {
    pub scroll_v: u16,
    pub scroll_h: u16,
    pub search: String,
    pub search_active: bool,
    pub match_indices: Vec<usize>,
    pub current_match: usize,
}

impl ConfigPanelState {
    /// Create a new ConfigPanelState with default values
    pub fn new() -> Self {
        Self::default()
    }

    /// Reset scroll and search state (call when changing runs)
    pub fn reset(&mut self) {
        self.scroll_v = 0;
        self.scroll_h = 0;
        self.search.clear();
        self.search_active = false;
        self.match_indices.clear();
        self.current_match = 0;
    }
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
                    .is_some_and(|&m| m == idx);

                if !self.state.search.is_empty() && is_match_line {
                    // Highlight matching text (Unicode-safe)
                    let matches = case_insensitive_byte_ranges(line, &self.state.search);
                    let mut spans = Vec::new();
                    let mut last_end = 0;

                    for (start, end) in matches {
                        if start > last_end {
                            spans.push(Span::raw(&line[last_end..start]));
                        }
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
