//! Help overlay widget showing keyboard shortcuts.

use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
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
            (
                "Navigation",
                vec![
                    ("↑/↓", "Move up/down in list"),
                    ("←/→", "Scroll config left/right"),
                    ("Esc", "Go back / clear search"),
                    ("Tab", "Cycle focus between panels"),
                    ("Shift+Tab", "Cycle focus backwards"),
                ],
            ),
            (
                "Metrics",
                vec![
                    ("1-9", "Select metric slot"),
                    ("[ / ]", "Shift metric window"),
                ],
            ),
            (
                "Comparison",
                vec![
                    ("s", "Toggle run for comparison"),
                    ("S", "Clear all comparisons"),
                ],
            ),
            (
                "Config Search",
                vec![
                    ("/", "Search config"),
                    ("n / N", "Next/previous match"),
                    ("c", "Clear search"),
                ],
            ),
            (
                "General",
                vec![
                    ("h / ?", "Toggle this help"),
                    ("r", "Refresh data"),
                    ("q", "Quit"),
                ],
            ),
        ];

        // Styles
        let desc_style = Style::default().fg(Color::Gray);
        let section_style = Style::default()
            .fg(Color::Cyan)
            .add_modifier(Modifier::BOLD);
        let key_style = Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD);
        let action_style = Style::default().fg(Color::White);
        let border_style = Style::default().fg(Color::Cyan);
        let title_style = Style::default()
            .fg(Color::Cyan)
            .add_modifier(Modifier::BOLD);

        let mut lines: Vec<Line> = Vec::new();

        // Description
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            format!("  {DESCRIPTION}"),
            desc_style,
        )));
        lines.push(Line::from(""));

        for (section, items) in shortcuts {
            // Section header
            lines.push(Line::from(Span::styled(
                format!("  {section}"),
                section_style,
            )));
            lines.push(Line::from(""));

            for (key, desc) in items {
                lines.push(Line::from(vec![
                    Span::raw("    "),
                    Span::styled(format!("{key:<14}"), key_style),
                    Span::styled(desc, action_style),
                ]));
            }
            lines.push(Line::from(""));
        }

        let paragraph = Paragraph::new(lines)
            .block(
                Block::default()
                    .title(" trackio-tui Help ")
                    .title_alignment(Alignment::Center)
                    .title_style(title_style)
                    .borders(Borders::ALL)
                    .border_style(border_style),
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
