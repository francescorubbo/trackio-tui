//! UI widgets for the trackio dashboard.

use ratatui::{
    layout::Rect,
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Wrap},
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

    pub fn render(&self, frame: &mut Frame, area: Rect) {
        let items: Vec<ListItem> = self.projects
            .iter()
            .map(|p| ListItem::new(format!("{} ({})", p.name, p.run_count)))
            .collect();

        let list = List::new(items)
            .block(Block::default().title(" Projects ").borders(Borders::ALL))
            .highlight_symbol("> ");

        let mut state = ListState::default();
        state.select(Some(self.selected));
        frame.render_stateful_widget(list, area, &mut state);
    }
}

/// Run list panel widget
pub struct RunList<'a> {
    runs: &'a [Run],
    selected: usize,
}

impl<'a> RunList<'a> {
    pub fn new(runs: &'a [Run], selected: usize) -> Self {
        RunList { runs, selected }
    }

    pub fn render(&self, frame: &mut Frame, area: Rect) {
        let items: Vec<ListItem> = self.runs
            .iter()
            .map(|r| ListItem::new(format!("{} [{}]", r.display_name(), r.status.display())))
            .collect();

        let list = List::new(items)
            .block(Block::default().title(format!(" Runs ({}) ", self.runs.len())).borders(Borders::ALL))
            .highlight_symbol("> ");

        let mut state = ListState::default();
        state.select(Some(self.selected));
        frame.render_stateful_widget(list, area, &mut state);
    }
}

/// Config panel widget
pub struct ConfigPanel<'a> {
    config: &'a [Config],
}

impl<'a> ConfigPanel<'a> {
    pub fn new(config: &'a [Config]) -> Self {
        ConfigPanel { config }
    }

    pub fn render(&self, frame: &mut Frame, area: Rect) {
        let text: String = self.config
            .iter()
            .map(|c| format!("{}: {}", c.key, c.value))
            .collect::<Vec<_>>()
            .join("\n");

        let paragraph = Paragraph::new(text)
            .block(Block::default().title(" Config ").borders(Borders::ALL))
            .wrap(Wrap { trim: true });

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
            format!("Error: {}", e)
        } else {
            match self.project {
                Some(p) => format!("trackio-tui: {} | [h] Help [q] Quit", p),
                None => "trackio-tui | [h] Help [q] Quit".to_string(),
            }
        };

        let paragraph = Paragraph::new(text)
            .block(Block::default().borders(Borders::TOP));

        frame.render_widget(paragraph, area);
    }
}
