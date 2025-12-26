//! Main application logic and TUI event loop.

use std::collections::HashMap;
use std::io;
use std::time::{Duration, Instant};

use anyhow::{Context, Result};
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    Terminal,
};

use crate::cli::AppConfig;
use crate::data::{Config, Metric, Project, Run, Storage};
use crate::ui::{
    chart::{ChartConfig, MetricSelector, MetricsChart},
    HelpOverlay, Theme,
    widgets::{ConfigPanel, ProjectList, RunList, StatusBar},
};

/// Manages run comparison state and cached metrics
#[derive(Debug, Default)]
pub struct ComparisonState {
    /// Run indices marked for comparison
    marked_runs: Vec<usize>,
    /// Cached metrics for comparison, keyed by run index
    metrics_cache: HashMap<usize, Vec<Metric>>,
}

impl ComparisonState {
    /// Create a new empty comparison state
    pub fn new() -> Self {
        Self::default()
    }

    /// Toggle a run's comparison status. Returns true if run is now marked.
    pub fn toggle_run(&mut self, run_idx: usize) -> bool {
        if let Some(pos) = self.marked_runs.iter().position(|&r| r == run_idx) {
            self.marked_runs.remove(pos);
            self.metrics_cache.remove(&run_idx);
            false
        } else {
            self.marked_runs.push(run_idx);
            true
        }
    }

    /// Check if a run is marked for comparison
    #[allow(dead_code)] // Used in tests
    pub fn is_marked(&self, run_idx: usize) -> bool {
        self.marked_runs.contains(&run_idx)
    }

    /// Get the list of marked run indices
    pub fn marked_runs(&self) -> &[usize] {
        &self.marked_runs
    }

    /// Clear all comparison state
    pub fn clear(&mut self) {
        self.marked_runs.clear();
        self.metrics_cache.clear();
    }

    /// Cache metrics for a run
    pub fn cache_metrics(&mut self, run_idx: usize, metrics: Vec<Metric>) {
        self.metrics_cache.insert(run_idx, metrics);
    }

    /// Get cached metrics for a specific run
    #[allow(dead_code)] // Used in tests
    pub fn get_cached_metrics(&self, run_idx: usize) -> Option<&Vec<Metric>> {
        self.metrics_cache.get(&run_idx)
    }

    /// Get comparison metrics for display, excluding the currently selected run.
    /// Returns an iterator of (run_idx, metric) pairs.
    pub fn get_comparison_metrics(&self, selected_run: usize) -> impl Iterator<Item = (usize, &Metric)> {
        self.marked_runs
            .iter()
            .filter(move |&&run_idx| run_idx != selected_run)
            .flat_map(|&run_idx| {
                self.metrics_cache
                    .get(&run_idx)
                    .into_iter()
                    .flat_map(move |metrics| metrics.iter().map(move |m| (run_idx, m)))
            })
    }

    /// Check if there are any runs marked for comparison
    #[allow(dead_code)] // Used in tests
    pub fn has_comparisons(&self) -> bool {
        !self.marked_runs.is_empty()
    }

    /// Remove runs that are no longer valid (index out of bounds)
    pub fn prune_invalid_runs(&mut self, max_run_idx: usize) {
        self.marked_runs.retain(|&run_idx| run_idx < max_run_idx);
        self.metrics_cache.retain(|&run_idx, _| run_idx < max_run_idx);
    }
}

/// Which panel is currently focused
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FocusedPanel {
    Projects,
    Runs,
    Metrics,
}

impl FocusedPanel {
    fn next(self) -> Self {
        match self {
            FocusedPanel::Projects => FocusedPanel::Runs,
            FocusedPanel::Runs => FocusedPanel::Metrics,
            FocusedPanel::Metrics => FocusedPanel::Projects,
        }
    }

    fn prev(self) -> Self {
        match self {
            FocusedPanel::Projects => FocusedPanel::Metrics,
            FocusedPanel::Runs => FocusedPanel::Projects,
            FocusedPanel::Metrics => FocusedPanel::Runs,
        }
    }
}

/// Application state
pub struct App {
    // Configuration
    config: AppConfig,
    theme: Theme,
    
    // Data
    storage: Storage,
    projects: Vec<Project>,
    runs: Vec<Run>,
    metrics: Vec<Metric>,
    metric_names: Vec<String>,
    current_config: Vec<Config>,
    
    // Comparison state
    comparison: ComparisonState,
    
    // UI State
    focused: FocusedPanel,
    selected_project: usize,
    selected_run: usize,
    selected_metric: usize,
    show_help: bool,
    
    // Chart settings
    chart_config: ChartConfig,
    
    // Timing
    last_refresh: Instant,
    
    // Exit flag
    should_quit: bool,
    
    // Error message to display (non-fatal)
    error_message: Option<String>,
}

impl App {
    /// Create a new App instance
    pub fn new(config: AppConfig) -> Result<Self> {
        let theme = Theme::default();
        let storage = Storage::new(config.db_path.clone());
        
        let mut app = App {
            config,
            theme,
            storage,
            projects: Vec::new(),
            runs: Vec::new(),
            metrics: Vec::new(),
            metric_names: Vec::new(),
            current_config: Vec::new(),
            comparison: ComparisonState::new(),
            focused: FocusedPanel::Projects,
            selected_project: 0,
            selected_run: 0,
            selected_metric: 0,
            show_help: false,
            chart_config: ChartConfig::default(),
            last_refresh: Instant::now(),
            should_quit: false,
            error_message: None,
        };
        
        // Initial data load
        app.load_projects()?;
        
        // If a project was specified, select it
        if let Some(ref project_name) = app.config.project {
            if let Some(idx) = app.projects.iter().position(|p| &p.name == project_name) {
                app.selected_project = idx;
                app.load_runs()?;
            }
        } else if !app.projects.is_empty() {
            app.load_runs()?;
        }
        
        Ok(app)
    }
    
    /// Load list of projects from storage
    fn load_projects(&mut self) -> Result<()> {
        self.projects = self.storage.list_projects()?;
        if self.selected_project >= self.projects.len() {
            self.selected_project = self.projects.len().saturating_sub(1);
        }
        Ok(())
    }
    
    /// Load runs for the currently selected project.
    /// If `clear_comparison` is true, clears comparison state (used on project change).
    fn load_runs_impl(&mut self, clear_comparison: bool) -> Result<()> {
        if self.projects.is_empty() {
            self.runs.clear();
            self.metrics.clear();
            self.metric_names.clear();
            self.current_config.clear();
            self.comparison.clear();
            return Ok(());
        }
        
        let project = &self.projects[self.selected_project];
        self.runs = self.storage.list_runs(&project.name)?;
        
        if self.selected_run >= self.runs.len() {
            self.selected_run = self.runs.len().saturating_sub(1);
        }
        
        if clear_comparison {
            // Clear comparison selection when changing projects
            self.comparison.clear();
        } else {
            // Prune any invalid run indices after refresh
            self.comparison.prune_invalid_runs(self.runs.len());
        }
        
        // Load metrics for selected run
        self.load_metrics()?;
        
        Ok(())
    }
    
    /// Load runs for the currently selected project, clearing comparison state.
    fn load_runs(&mut self) -> Result<()> {
        self.load_runs_impl(true)
    }
    
    /// Reload runs without clearing comparison state (for refresh).
    fn reload_runs(&mut self) -> Result<()> {
        self.load_runs_impl(false)
    }
    
    /// Load metrics for the currently selected run
    fn load_metrics(&mut self) -> Result<()> {
        if self.runs.is_empty() {
            self.metrics.clear();
            self.metric_names.clear();
            self.current_config.clear();
            return Ok(());
        }
        
        let project = &self.projects[self.selected_project];
        let run = &self.runs[self.selected_run];
        
        self.metric_names = self.storage.list_metrics(&project.name, &run.id)?;
        self.current_config = run.config.clone();
        
        // Load all metrics data
        self.metrics = self.storage.get_all_metrics(&project.name, &run.id)?;
        
        if self.selected_metric >= self.metric_names.len() {
            self.selected_metric = self.metric_names.len().saturating_sub(1);
        }
        
        Ok(())
    }
    
    /// Refresh all data without clearing comparison state
    fn refresh(&mut self) -> Result<()> {
        self.error_message = None; // Clear any previous errors
        self.load_projects()?;
        self.reload_runs()?; // Use reload_runs to preserve comparison state
        self.load_comparison_metrics()?;
        self.last_refresh = Instant::now();
        Ok(())
    }
    
    /// Set an error message to display (non-fatal)
    pub fn set_error(&mut self, message: String) {
        self.error_message = Some(message);
    }
    
    /// Load/refresh metrics for all comparison runs into the cache
    fn load_comparison_metrics(&mut self) -> Result<()> {
        if self.projects.is_empty() || self.runs.is_empty() {
            return Ok(());
        }
        
        let project = &self.projects[self.selected_project];
        
        for &run_idx in self.comparison.marked_runs().to_vec().iter() {
            if run_idx >= self.runs.len() {
                continue;
            }
            
            let run = &self.runs[run_idx];
            if let Ok(metrics) = self.storage.get_all_metrics(&project.name, &run.id) {
                self.comparison.cache_metrics(run_idx, metrics);
            }
        }
        
        Ok(())
    }
    
    /// Load metrics for a single comparison run into the cache
    fn load_single_comparison_run(&mut self, run_idx: usize) -> Result<()> {
        if self.projects.is_empty() || run_idx >= self.runs.len() {
            return Ok(());
        }
        
        let project = &self.projects[self.selected_project];
        let run = &self.runs[run_idx];
        
        if let Ok(metrics) = self.storage.get_all_metrics(&project.name, &run.id) {
            self.comparison.cache_metrics(run_idx, metrics);
        }
        
        Ok(())
    }
    
    /// Handle keyboard input
    fn handle_input(&mut self, key: KeyCode, _modifiers: KeyModifiers) -> Result<()> {
        // Global shortcuts
        match key {
            KeyCode::Char('q') => {
                self.should_quit = true;
                return Ok(());
            }
            KeyCode::Char('?') | KeyCode::Char('h') | KeyCode::F(1) => {
                self.show_help = !self.show_help;
                return Ok(());
            }
            KeyCode::Esc if self.show_help => {
                self.show_help = false;
                return Ok(());
            }
            KeyCode::Char('r') => {
                self.refresh()?;
                return Ok(());
            }
            KeyCode::Tab => {
                self.focused = self.focused.next();
                return Ok(());
            }
            KeyCode::BackTab => {
                self.focused = self.focused.prev();
                return Ok(());
            }
            _ => {}
        }
        
        // If help is shown, don't process other keys
        if self.show_help {
            return Ok(());
        }
        
        // Metric selection with number keys
        if let KeyCode::Char(c) = key {
            if let Some(n) = c.to_digit(10) {
                if n > 0 && (n as usize) <= self.metric_names.len() {
                    self.selected_metric = (n as usize) - 1;
                    return Ok(());
                }
            }
        }
        
        // Smoothing controls
        match key {
            KeyCode::Char('+') | KeyCode::Char('=') => {
                self.chart_config.smoothing = (self.chart_config.smoothing + 0.05).min(0.99);
                return Ok(());
            }
            KeyCode::Char('-') => {
                self.chart_config.smoothing = (self.chart_config.smoothing - 0.05).max(0.0);
                return Ok(());
            }
            _ => {}
        }
        
        // X-axis controls
        match key {
            KeyCode::Char('[') => {
                // Zoom out (show more data)
                if let Some(min) = self.chart_config.x_min {
                    self.chart_config.x_min = Some(min - 50.0);
                }
                return Ok(());
            }
            KeyCode::Char(']') => {
                // Zoom in (show less data)
                if self.chart_config.x_min.is_none() {
                    // Initialize zoom
                    if let Some(metric) = self.metrics.first() {
                        if let Some((_, max)) = metric.step_range() {
                            self.chart_config.x_min = Some((max as f64 - 100.0).max(0.0));
                        }
                    }
                } else if let Some(min) = self.chart_config.x_min {
                    self.chart_config.x_min = Some((min + 50.0).max(0.0));
                }
                return Ok(());
            }
            _ => {}
        }
        
        // Toggle run for comparison
        if key == KeyCode::Char('S') {
            // Shift+S: Clear all comparison selections
            self.comparison.clear();
            return Ok(());
        }
        if key == KeyCode::Char('s') && self.focused == FocusedPanel::Runs {
            // s: Toggle current run in comparison
            let was_added = self.comparison.toggle_run(self.selected_run);
            if was_added {
                // Load metrics for the newly added run
                self.load_single_comparison_run(self.selected_run)?;
            }
            return Ok(());
        }
        
        // Panel-specific navigation
        match self.focused {
            FocusedPanel::Projects => self.handle_project_navigation(key)?,
            FocusedPanel::Runs => self.handle_run_navigation(key)?,
            FocusedPanel::Metrics => self.handle_metric_navigation(key)?,
        }
        
        Ok(())
    }
    
    fn handle_project_navigation(&mut self, key: KeyCode) -> Result<()> {
        match key {
            KeyCode::Down | KeyCode::Char('j') => {
                if !self.projects.is_empty() {
                    self.selected_project = (self.selected_project + 1) % self.projects.len();
                    self.load_runs()?;
                }
            }
            KeyCode::Up | KeyCode::Char('k') => {
                if !self.projects.is_empty() {
                    self.selected_project = self.selected_project
                        .checked_sub(1)
                        .unwrap_or(self.projects.len() - 1);
                    self.load_runs()?;
                }
            }
            KeyCode::Enter | KeyCode::Char('l') => {
                self.focused = FocusedPanel::Runs;
            }
            _ => {}
        }
        Ok(())
    }
    
    fn handle_run_navigation(&mut self, key: KeyCode) -> Result<()> {
        match key {
            KeyCode::Down | KeyCode::Char('j') => {
                if !self.runs.is_empty() {
                    self.selected_run = (self.selected_run + 1) % self.runs.len();
                    self.load_metrics()?;
                }
            }
            KeyCode::Up | KeyCode::Char('k') => {
                if !self.runs.is_empty() {
                    self.selected_run = self.selected_run
                        .checked_sub(1)
                        .unwrap_or(self.runs.len() - 1);
                    self.load_metrics()?;
                }
            }
            KeyCode::Enter | KeyCode::Char('l') => {
                self.focused = FocusedPanel::Metrics;
            }
            KeyCode::Esc => {
                self.focused = FocusedPanel::Projects;
            }
            _ => {}
        }
        Ok(())
    }
    
    fn handle_metric_navigation(&mut self, key: KeyCode) -> Result<()> {
        match key {
            KeyCode::Down | KeyCode::Char('j') | KeyCode::Right | KeyCode::Char('l') => {
                if !self.metric_names.is_empty() {
                    self.selected_metric = (self.selected_metric + 1) % self.metric_names.len();
                }
            }
            KeyCode::Up | KeyCode::Char('k') | KeyCode::Left => {
                if !self.metric_names.is_empty() {
                    self.selected_metric = self.selected_metric
                        .checked_sub(1)
                        .unwrap_or(self.metric_names.len() - 1);
                }
            }
            KeyCode::Esc => {
                self.focused = FocusedPanel::Runs;
            }
            _ => {}
        }
        Ok(())
    }
    
    /// Render the UI
    fn render(&self, frame: &mut ratatui::Frame) {
        let size = frame.area();
        
        // Main layout: header, body, footer
        let main_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(3),    // Body
                Constraint::Length(2), // Status bar
            ])
            .split(size);
        
        // Body layout: sidebar (left) and content (right)
        let body_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Length(25), // Sidebar
                Constraint::Min(40),    // Content
            ])
            .split(main_chunks[0]);
        
        // Sidebar layout: projects, runs, config
        let sidebar_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage(30), // Projects
                Constraint::Percentage(40), // Runs
                Constraint::Percentage(30), // Config
            ])
            .split(body_chunks[0]);
        
        // Content layout: chart and metric selector
        let content_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(10),   // Chart
                Constraint::Length(1), // Metric selector
            ])
            .split(body_chunks[1]);
        
        // Render sidebar components
        let project_list = ProjectList::new(
            &self.projects,
            self.selected_project,
            &self.theme,
        );
        project_list.render(frame, sidebar_chunks[0], self.focused == FocusedPanel::Projects);
        
        let run_list = RunList::new(
            &self.runs,
            self.selected_run,
            self.comparison.marked_runs(),
            &self.theme,
        );
        run_list.render(frame, sidebar_chunks[1], self.focused == FocusedPanel::Runs);
        
        let config_panel = ConfigPanel::new(&self.current_config, &self.theme);
        config_panel.render(frame, sidebar_chunks[2], false);
        
        // Render chart
        let current_metric_name = self.metric_names.get(self.selected_metric)
            .map(|s| s.as_str())
            .unwrap_or("No metric selected");
        
        // Gather metrics for display (including comparison runs)
        let mut chart_metrics: Vec<(String, &Metric)> = Vec::new();
        
        // Add current run's metric
        if let Some(metric) = self.metrics.iter()
            .find(|m| m.name == current_metric_name) {
            let run_name = self.runs.get(self.selected_run)
                .map(|r| r.display_name())
                .unwrap_or_default();
            chart_metrics.push((run_name, metric));
        }
        
        // Add comparison runs' metrics (excludes currently selected run)
        for (run_idx, metric) in self.comparison.get_comparison_metrics(self.selected_run) {
            if metric.name == current_metric_name {
                if let Some(run) = self.runs.get(run_idx) {
                    chart_metrics.push((format!("{}*", run.display_name()), metric));
                }
            }
        }
        
        let chart = MetricsChart::new(
            &chart_metrics,
            current_metric_name,
            &self.chart_config,
            &self.theme,
        );
        chart.render(frame, content_chunks[0], self.focused == FocusedPanel::Metrics);
        
        // Render metric selector
        let metric_selector = MetricSelector::new(
            &self.metric_names,
            self.selected_metric,
            &self.theme,
        );
        metric_selector.render(frame, content_chunks[1]);
        
        // Render status bar
        let project_name = self.projects.get(self.selected_project)
            .map(|p| p.name.as_str());
        let error_msg = self.error_message.as_deref();
        let status_bar = StatusBar::new(
            project_name,
            Some(current_metric_name),
            self.chart_config.smoothing,
            error_msg,
            &self.theme,
        );
        status_bar.render(frame, main_chunks[1]);
        
        // Render help overlay if active
        if self.show_help {
            let help = HelpOverlay::new(&self.theme);
            help.render(frame, size);
        }
    }
}

/// Restore terminal to normal state
fn restore_terminal() {
    // Best effort cleanup - ignore errors since we may be in a panic
    let _ = disable_raw_mode();
    let _ = execute!(io::stdout(), LeaveAlternateScreen, DisableMouseCapture);
}

/// Run the TUI application
pub fn run(config: AppConfig) -> Result<()> {
    // Check if database exists
    let db_path = &config.db_path;
    if !db_path.exists() {
        eprintln!("No trackio data found at: {db_path:?}");
        eprintln!("Run some experiments with trackio first, or specify a different path with --db-path");
        return Ok(());
    }
    
    // Setup terminal
    enable_raw_mode().context("Failed to enable raw mode")?;
    let mut stdout = io::stdout();
    if let Err(e) = execute!(stdout, EnterAlternateScreen, EnableMouseCapture) {
        restore_terminal();
        return Err(e).context("Failed to setup terminal");
    }
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = match Terminal::new(backend) {
        Ok(t) => t,
        Err(e) => {
            restore_terminal();
            return Err(e).context("Failed to create terminal");
        }
    };
    
    // Create app - if this fails, restore terminal first
    let mut app = match App::new(config) {
        Ok(a) => a,
        Err(e) => {
            restore_terminal();
            return Err(e).context("Failed to initialize application");
        }
    };
    let tick_rate = Duration::from_secs(app.config.refresh_interval_secs);
    
    // Main loop - wrap in a closure to ensure cleanup
    let result = run_main_loop(&mut terminal, &mut app, tick_rate);
    
    // Always restore terminal, regardless of result
    restore_terminal();
    terminal.show_cursor().ok();
    
    result
}

/// Main application loop
fn run_main_loop(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    app: &mut App,
    tick_rate: Duration,
) -> Result<()> {
    loop {
        // Render - if this fails, we should exit
        terminal.draw(|f| app.render(f))?;
        
        // Check if it's time to refresh (ignore refresh errors, just continue)
        if app.last_refresh.elapsed() >= tick_rate {
            if let Err(e) = app.refresh() {
                // Log error but don't crash
                app.set_error(format!("Refresh error: {e}"));
            }
        }
        
        // Handle input with timeout
        let timeout = tick_rate.saturating_sub(app.last_refresh.elapsed());
        if event::poll(timeout.min(Duration::from_millis(100)))? {
            if let Event::Key(key) = event::read()? {
                if let Err(e) = app.handle_input(key.code, key.modifiers) {
                    // Log error but don't crash
                    app.set_error(format!("Input error: {e}"));
                }
            }
        }
        
        if app.should_quit {
            return Ok(());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::{Metric, MetricPoint};

    fn make_metric(name: &str, values: &[f64]) -> Metric {
        Metric {
            name: name.to_string(),
            points: values
                .iter()
                .enumerate()
                .map(|(i, &v)| MetricPoint {
                    step: i as i64,
                    value: v,
                    timestamp: None,
                })
                .collect(),
        }
    }

    #[test]
    fn test_toggle_run_adds_and_removes() {
        let mut state = ComparisonState::new();

        // Initially empty
        assert!(!state.is_marked(0));
        assert!(state.marked_runs().is_empty());

        // Toggle on
        let added = state.toggle_run(0);
        assert!(added);
        assert!(state.is_marked(0));
        assert_eq!(state.marked_runs(), &[0]);

        // Toggle off
        let added = state.toggle_run(0);
        assert!(!added);
        assert!(!state.is_marked(0));
        assert!(state.marked_runs().is_empty());
    }

    #[test]
    fn test_multiple_runs_marked() {
        let mut state = ComparisonState::new();

        state.toggle_run(1);
        state.toggle_run(3);
        state.toggle_run(5);

        assert!(!state.is_marked(0));
        assert!(state.is_marked(1));
        assert!(!state.is_marked(2));
        assert!(state.is_marked(3));
        assert!(!state.is_marked(4));
        assert!(state.is_marked(5));
        assert_eq!(state.marked_runs(), &[1, 3, 5]);
    }

    #[test]
    fn test_clear_removes_all() {
        let mut state = ComparisonState::new();

        state.toggle_run(0);
        state.toggle_run(1);
        state.cache_metrics(0, vec![make_metric("loss", &[1.0, 0.5])]);
        state.cache_metrics(1, vec![make_metric("loss", &[0.9, 0.4])]);

        assert!(state.has_comparisons());
        assert!(state.get_cached_metrics(0).is_some());

        state.clear();

        assert!(!state.has_comparisons());
        assert!(state.marked_runs().is_empty());
        assert!(state.get_cached_metrics(0).is_none());
        assert!(state.get_cached_metrics(1).is_none());
    }

    #[test]
    fn test_get_comparison_metrics_excludes_selected() {
        let mut state = ComparisonState::new();

        // Mark runs 0, 1, 2
        state.toggle_run(0);
        state.toggle_run(1);
        state.toggle_run(2);

        state.cache_metrics(0, vec![make_metric("loss", &[1.0])]);
        state.cache_metrics(1, vec![make_metric("loss", &[0.9])]);
        state.cache_metrics(2, vec![make_metric("loss", &[0.8])]);

        // When selected_run is 1, we should get metrics for runs 0 and 2 only
        let comparison: Vec<(usize, &Metric)> = state.get_comparison_metrics(1).collect();
        
        assert_eq!(comparison.len(), 2);
        assert!(comparison.iter().any(|(idx, _)| *idx == 0));
        assert!(comparison.iter().any(|(idx, _)| *idx == 2));
        assert!(!comparison.iter().any(|(idx, _)| *idx == 1));
    }

    #[test]
    fn test_get_comparison_metrics_with_multiple_metrics_per_run() {
        let mut state = ComparisonState::new();

        state.toggle_run(0);
        state.cache_metrics(
            0,
            vec![
                make_metric("loss", &[1.0, 0.5]),
                make_metric("accuracy", &[0.5, 0.8]),
            ],
        );

        // When selected is 1, we get all metrics from run 0
        let comparison: Vec<(usize, &Metric)> = state.get_comparison_metrics(1).collect();
        
        assert_eq!(comparison.len(), 2);
        assert!(comparison.iter().all(|(idx, _)| *idx == 0));
        
        let names: Vec<&str> = comparison.iter().map(|(_, m)| m.name.as_str()).collect();
        assert!(names.contains(&"loss"));
        assert!(names.contains(&"accuracy"));
    }

    #[test]
    fn test_prune_invalid_runs() {
        let mut state = ComparisonState::new();

        state.toggle_run(0);
        state.toggle_run(3);
        state.toggle_run(5);
        state.cache_metrics(0, vec![make_metric("loss", &[1.0])]);
        state.cache_metrics(3, vec![make_metric("loss", &[0.9])]);
        state.cache_metrics(5, vec![make_metric("loss", &[0.8])]);

        // Prune to max index 4 (runs 0, 1, 2, 3 are valid; 5 is invalid)
        state.prune_invalid_runs(4);

        assert!(state.is_marked(0));
        assert!(state.is_marked(3));
        assert!(!state.is_marked(5));
        assert!(state.get_cached_metrics(0).is_some());
        assert!(state.get_cached_metrics(3).is_some());
        assert!(state.get_cached_metrics(5).is_none());
    }

    #[test]
    fn test_toggle_removes_cached_metrics() {
        let mut state = ComparisonState::new();

        state.toggle_run(0);
        state.cache_metrics(0, vec![make_metric("loss", &[1.0])]);
        assert!(state.get_cached_metrics(0).is_some());

        // Toggle off should also remove cached metrics
        state.toggle_run(0);
        assert!(state.get_cached_metrics(0).is_none());
    }

    #[test]
    fn test_has_comparisons() {
        let mut state = ComparisonState::new();

        assert!(!state.has_comparisons());

        state.toggle_run(0);
        assert!(state.has_comparisons());

        state.toggle_run(0);
        assert!(!state.has_comparisons());
    }
}

