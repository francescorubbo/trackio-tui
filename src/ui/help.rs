//! Help overlay widget showing keyboard shortcuts.

use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
    Frame,
};

/// Help overlay showing all keyboard shortcuts
pub struct HelpOverlay;

impl HelpOverlay {
    pub fn new() -> Self {
        HelpOverlay
    }

    pub fn render(&self, frame: &mut Frame, area: Rect) {
        let popup_area = centered_rect(65, 80, area);
        frame.render_widget(Clear, popup_area);

        const DESCRIPTION: &str = "A terminal dashboard for visualizing machine learning experiments tracked with trackio. Browse projects, compare runs, and monitor metrics in real-time.";

        let shortcuts = [
            ("Navigation", vec![
                ("j / ↓", "Move down in list"),
                ("k / ↑", "Move up in list"),
                ("Enter / l", "Select item / move right"),
                ("Esc", "Go back / move left"),
                ("Tab", "Cycle focus between panels"),
                ("Shift+Tab", "Cycle focus backwards"),
            ]),
            ("Metrics", vec![
                ("1-9", "Select metric by number"),
            ]),
            ("Comparison", vec![
                ("s", "Toggle run for comparison"),
                ("S", "Clear all comparisons"),
            ]),
            ("General", vec![
                ("r", "Refresh data"),
                ("h / ?", "Toggle this help"),
                ("q", "Quit"),
            ]),
        ];

        let mut text = format!("\n  {DESCRIPTION}\n\n");

        for (section, items) in shortcuts {
            text.push_str(&format!("  {section}\n\n"));
            for (key, desc) in items {
                text.push_str(&format!("    {key:<14}{desc}\n"));
            }
            text.push('\n');
        }

        let paragraph = Paragraph::new(text)
            .block(
                Block::default()
                    .title(" trackio-tui Help ")
                    .title_alignment(Alignment::Center)
                    .borders(Borders::ALL),
            )
            .wrap(Wrap { trim: false });

        frame.render_widget(paragraph, popup_area);
    }
}

/// Create a centered rect for popup dialogs
fn centered_rect(percent_x: u16, percent_y: u16, area: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(area);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}
