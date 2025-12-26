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
            // Using named colors instead of RGB for better terminal compatibility
            chart_colors: vec![
                Color::Red,
                Color::Green,
                Color::Yellow,
                Color::Blue,
                Color::Magenta,
                Color::Cyan,
                Color::LightRed,
                Color::LightGreen,
            ],
        }
    }
}

impl Theme {
    /// Base surface style used to paint widget backgrounds
    pub fn surface_style(&self) -> Style {
        Style::default().fg(self.fg).bg(self.bg)
    }

    /// Convenience helper returning (border_style, title_style) for focus state
    pub fn panel_styles(&self, focused: bool) -> (Style, Style) {
        if focused {
            (self.focused_border_style(), self.focused_border_style())
        } else {
            (self.border_style(), self.dimmed_title_style())
        }
    }

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

    /// Get style for focused panel borders (distinct from normal borders)
    pub fn focused_border_style(&self) -> Style {
        Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD)
    }

    /// Get style for titles
    pub fn title_style(&self) -> Style {
        Style::default()
            .fg(self.title)
            .add_modifier(Modifier::BOLD)
    }

    /// Get style for unfocused/dimmed titles
    pub fn dimmed_title_style(&self) -> Style {
        Style::default()
            .fg(self.border)
            .add_modifier(Modifier::DIM)
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chart_colors_are_distinct() {
        let theme = Theme::default();
        // Verify first few colors are all different
        let c0 = theme.chart_color(0);
        let c1 = theme.chart_color(1);
        let c2 = theme.chart_color(2);
        assert_ne!(c0, c1, "Colors 0 and 1 should be different");
        assert_ne!(c1, c2, "Colors 1 and 2 should be different");
        assert_ne!(c0, c2, "Colors 0 and 2 should be different");
    }

    #[test]
    fn test_chart_color_cycles() {
        let theme = Theme::default();
        let len = theme.chart_colors.len();
        // Color at index 0 should equal color at index len (cycle)
        assert_eq!(theme.chart_color(0), theme.chart_color(len));
        assert_eq!(theme.chart_color(1), theme.chart_color(len + 1));
    }

    #[test]
    fn test_chart_colors_are_not_gray() {
        let theme = Theme::default();
        for (i, color) in theme.chart_colors.iter().enumerate() {
            // Verify none of the colors are gray shades
            let gray_colors = [Color::Gray, Color::DarkGray, Color::White, Color::Black];
            assert!(
                !gray_colors.contains(color),
                "Chart color {i} should not be a gray shade: {:?}",
                color
            );
        }
    }
}
