//! Main application logic and TUI event loop.

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
use crate::comparison::ComparisonState;
use crate::data::{Config, Metric, Project, Run, Storage};
use crate::ui::{
    chart::{MetricSelector, MetricsChart},
    HelpOverlay, Theme,
    widgets::{ConfigPanel, ProjectList, RunList, StatusBar},
};

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
        // Tuple: (run_name, run_idx, metric)
        let mut chart_metrics: Vec<(String, usize, &Metric)> = Vec::new();
        
        // Add current run's metric
        if let Some(metric) = self.metrics.iter()
            .find(|m| m.name == current_metric_name) {
            let run_name = self.runs.get(self.selected_run)
                .map(|r| r.display_name())
                .unwrap_or_default();
            chart_metrics.push((run_name, self.selected_run, metric));
        }
        
        // Add comparison runs' metrics (excludes currently selected run)
        for (run_idx, metric) in self.comparison.get_comparison_metrics(self.selected_run) {
            if metric.name == current_metric_name {
                if let Some(run) = self.runs.get(run_idx) {
                    chart_metrics.push((run.display_name(), run_idx, metric));
                }
            }
        }
        
        let chart = MetricsChart::new(
            &chart_metrics,
            current_metric_name,
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
