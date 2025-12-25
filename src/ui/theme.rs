//! Theme configuration for the TUI.

use ratatui::style::{Color, Modifier, Style};

/// Color theme for the application
#[derive(Debug, Clone)]
pub struct Theme {
    #[allow(dead_code)]
    pub name: String,
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

impl Theme {
    /// Create a theme from a name
    pub fn from_name(name: &str) -> Self {
        match name.to_lowercase().as_str() {
            "soft" => Self::soft(),
            "dark" => Self::dark(),
            "light" => Self::light(),
            _ => Self::default(),
        }
    }

    /// Default dark theme
    pub fn default() -> Self {
        Theme {
            name: "default".to_string(),
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

    /// Soft muted theme
    pub fn soft() -> Self {
        Theme {
            name: "soft".to_string(),
            bg: Color::Rgb(30, 30, 40),
            fg: Color::Rgb(200, 200, 210),
            highlight_bg: Color::Rgb(50, 50, 70),
            highlight_fg: Color::Rgb(230, 230, 240),
            border: Color::Rgb(70, 70, 90),
            title: Color::Rgb(150, 180, 200),
            status_running: Color::Rgb(100, 200, 150),
            status_done: Color::Rgb(100, 150, 200),
            status_failed: Color::Rgb(200, 100, 100),
            chart_colors: vec![
                Color::Rgb(200, 150, 150),
                Color::Rgb(150, 200, 180),
                Color::Rgb(150, 170, 200),
                Color::Rgb(180, 200, 150),
                Color::Rgb(200, 200, 150),
                Color::Rgb(180, 150, 200),
            ],
        }
    }

    /// Dark theme with high contrast
    pub fn dark() -> Self {
        Theme {
            name: "dark".to_string(),
            bg: Color::Rgb(15, 15, 20),
            fg: Color::Rgb(220, 220, 230),
            highlight_bg: Color::Rgb(40, 40, 60),
            highlight_fg: Color::White,
            border: Color::Rgb(60, 60, 80),
            title: Color::Rgb(100, 200, 255),
            status_running: Color::Rgb(50, 255, 150),
            status_done: Color::Rgb(100, 180, 255),
            status_failed: Color::Rgb(255, 100, 100),
            chart_colors: vec![
                Color::Rgb(255, 100, 100),
                Color::Rgb(100, 255, 200),
                Color::Rgb(100, 200, 255),
                Color::Rgb(200, 255, 100),
                Color::Rgb(255, 255, 100),
                Color::Rgb(255, 100, 255),
            ],
        }
    }

    /// Light theme
    pub fn light() -> Self {
        Theme {
            name: "light".to_string(),
            bg: Color::Rgb(250, 250, 252),
            fg: Color::Rgb(30, 30, 40),
            highlight_bg: Color::Rgb(220, 225, 235),
            highlight_fg: Color::Rgb(20, 20, 30),
            border: Color::Rgb(180, 185, 195),
            title: Color::Rgb(50, 100, 150),
            status_running: Color::Rgb(50, 150, 80),
            status_done: Color::Rgb(60, 120, 180),
            status_failed: Color::Rgb(180, 60, 60),
            chart_colors: vec![
                Color::Rgb(200, 80, 80),
                Color::Rgb(60, 160, 140),
                Color::Rgb(60, 130, 180),
                Color::Rgb(120, 160, 60),
                Color::Rgb(200, 180, 60),
                Color::Rgb(160, 80, 160),
            ],
        }
    }

    /// Set chart colors from hex strings
    pub fn with_color_palette(mut self, colors: &[String]) -> Self {
        self.chart_colors = colors
            .iter()
            .filter_map(|hex| parse_hex_color(hex))
            .collect();
        
        // Ensure we have at least one color
        if self.chart_colors.is_empty() {
            self.chart_colors = Self::default().chart_colors;
        }
        
        self
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

/// Parse a hex color string like "#FF0000" to a Color
fn parse_hex_color(hex: &str) -> Option<Color> {
    let hex = hex.trim_start_matches('#');
    if hex.len() != 6 {
        return None;
    }

    let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
    let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
    let b = u8::from_str_radix(&hex[4..6], 16).ok()?;

    Some(Color::Rgb(r, g, b))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_hex_color() {
        assert_eq!(parse_hex_color("#FF0000"), Some(Color::Rgb(255, 0, 0)));
        assert_eq!(parse_hex_color("00FF00"), Some(Color::Rgb(0, 255, 0)));
        assert_eq!(parse_hex_color("#invalid"), None);
    }

    #[test]
    fn test_theme_from_name() {
        let theme = Theme::from_name("soft");
        assert_eq!(theme.name, "soft");

        let theme = Theme::from_name("unknown");
        assert_eq!(theme.name, "default");
    }
}

