//! Theme configuration for the TUI.

use ratatui::style::{Color, Modifier, Style};

/// Color theme for the application
#[derive(Debug, Clone)]
pub struct Theme {
    pub bg: Color,
    pub fg: Color,
    pub highlight_bg: Color,
    pub highlight_fg: Color,
    pub border: Color,
    pub title: Color,
    pub status_running: Color,
    pub status_done: Color,
    pub status_failed: Color,
    pub chart_colors: Vec<Color>,
}

impl Default for Theme {
    fn default() -> Self {
        Theme {
            bg: Color::Reset,
            fg: Color::White,
            highlight_bg: Color::Rgb(60, 60, 80),
            highlight_fg: Color::White,
            border: Color::Rgb(100, 100, 120),
            title: Color::Cyan,
            status_running: Color::Green,
            status_done: Color::Blue,
            status_failed: Color::Red,
            chart_colors: vec![
                Color::Rgb(255, 107, 107), // Red
                Color::Rgb(78, 205, 196),  // Teal
                Color::Rgb(69, 183, 209),  // Blue
                Color::Rgb(150, 206, 180), // Green
                Color::Rgb(255, 234, 167), // Yellow
                Color::Rgb(221, 160, 221), // Plum
                Color::Rgb(152, 216, 200), // Mint
                Color::Rgb(247, 220, 111), // Gold
            ],
        }
    }
}

impl Theme {
    /// Get style for normal text
    pub fn normal_style(&self) -> Style {
        Style::default().fg(self.fg).bg(self.bg)
    }

    /// Get style for highlighted/selected items
    pub fn highlight_style(&self) -> Style {
        Style::default()
            .fg(self.highlight_fg)
            .bg(self.highlight_bg)
            .add_modifier(Modifier::BOLD)
    }

    /// Get style for borders
    pub fn border_style(&self) -> Style {
        Style::default().fg(self.border)
    }

    /// Get style for titles
    pub fn title_style(&self) -> Style {
        Style::default()
            .fg(self.title)
            .add_modifier(Modifier::BOLD)
    }

    /// Get style for run status
    pub fn status_style(&self, status: &crate::data::RunStatus) -> Style {
        let color = match status {
            crate::data::RunStatus::Running => self.status_running,
            crate::data::RunStatus::Finished => self.status_done,
            crate::data::RunStatus::Failed => self.status_failed,
            crate::data::RunStatus::Unknown => self.fg,
        };
        Style::default().fg(color)
    }

    /// Get a chart color by index (cycles through available colors)
    pub fn chart_color(&self, index: usize) -> Color {
        self.chart_colors[index % self.chart_colors.len()]
    }
}
