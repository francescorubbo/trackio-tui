//! Help overlay widget showing keyboard shortcuts.

use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
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
        let popup_area = centered_rect(60, 70, area);

        // Clear the background
        frame.render_widget(Clear, popup_area);

        let help_items = vec![
            ("Navigation", vec![
                ("j / ↓", "Move down"),
                ("k / ↑", "Move up"),
                ("Enter / l", "Select / expand"),
                ("Esc / h", "Back / collapse"),
                ("Tab", "Cycle panels"),
            ]),
            ("Metrics", vec![
                ("1-9", "Select metric by number"),
                ("+ / -", "Adjust smoothing"),
                ("[ / ]", "Adjust x-axis range"),
            ]),
            ("Selection", vec![
                ("s", "Toggle run for comparison"),
                ("S", "Clear comparison selection"),
            ]),
            ("Other", vec![
                ("r", "Refresh data"),
                ("/", "Search / filter"),
                ("?", "Toggle this help"),
                ("q", "Quit"),
            ]),
        ];

        let mut lines: Vec<Line> = Vec::new();
        lines.push(Line::from(""));

        for (section, shortcuts) in help_items {
            lines.push(Line::from(Span::styled(
                format!("  {} ", section),
                Style::default()
                    .add_modifier(Modifier::BOLD)
                    .add_modifier(Modifier::UNDERLINED),
            )));
            lines.push(Line::from(""));

            for (key, desc) in shortcuts {
                lines.push(Line::from(vec![
                    Span::raw("    "),
                    Span::styled(
                        format!("{:<12}", key),
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
                    .title(" Keyboard Shortcuts ")
                    .title_alignment(Alignment::Center)
                    .borders(Borders::ALL)
                    .border_style(self.theme.border_style())
                    .title_style(self.theme.title_style())
                    .style(self.theme.normal_style()),
            )
            .alignment(Alignment::Left);

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

