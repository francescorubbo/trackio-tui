//! Help overlay widget showing keyboard shortcuts.

use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
    Frame,
};

use super::theme::Theme;

/// Help overlay showing all keyboard shortcuts
pub struct HelpOverlay<'a> {
    theme: &'a Theme,
}

impl<'a> HelpOverlay<'a> {
    pub fn new(theme: &'a Theme) -> Self {
        HelpOverlay { theme }
    }

    pub fn render(&self, frame: &mut Frame, area: Rect) {
        // Center the help popup
        let popup_area = centered_rect(65, 80, area);

        // Clear the background
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

        let mut lines: Vec<Line> = Vec::new();

        // Add description as a single wrapped line
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            format!("  {DESCRIPTION}"),
            Style::default().add_modifier(Modifier::ITALIC),
        )));
        lines.push(Line::from(""));

        // Add keyboard shortcuts sections
        for (section, items) in shortcuts {
            lines.push(Line::from(Span::styled(
                format!("  {section} "),
                Style::default()
                    .add_modifier(Modifier::BOLD)
                    .add_modifier(Modifier::UNDERLINED),
            )));
            lines.push(Line::from(""));

            for (key, desc) in items {
                lines.push(Line::from(vec![
                    Span::raw("    "),
                    Span::styled(
                        format!("{key:<14}"),
                        Style::default().fg(self.theme.title),
                    ),
                    Span::raw(desc),
                ]));
            }
            lines.push(Line::from(""));
        }

        let paragraph = Paragraph::new(lines)
            .block(
                Block::default()
                    .title(" trackio-tui Help ")
                    .title_alignment(Alignment::Center)
                    .borders(Borders::ALL)
                    .border_style(self.theme.border_style())
                    .title_style(self.theme.title_style())
                    .style(self.theme.surface_style()),
            )
            .alignment(Alignment::Left)
            .wrap(Wrap { trim: false })
            .style(self.theme.surface_style());

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
