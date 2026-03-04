//! # App
//!
//! Application state and event handling for the TUI.

use std::io;

use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{Terminal, backend::CrosstermBackend};
use tokio::sync::mpsc;

use crate::database::entities::Fight;
use crate::database::{Database, entities::Fighter};
use crate::engine::{
    Backtester, EloConfig, EventPrediction, OptimizationResult, Optimizer, ParameterRanges,
    get_upcoming_predictions,
};

use super::ui;

/// Weight class filter options.
pub const WEIGHT_CLASSES: &[(&str, &str)] = &[
    ("all", "All Classes"),
    ("heavyweight", "Heavyweight"),
    ("light-heavyweight", "Light Heavyweight"),
    ("middleweight", "Middleweight"),
    ("welterweight", "Welterweight"),
    ("lightweight", "Lightweight"),
    ("featherweight", "Featherweight"),
    ("bantamweight", "Bantamweight"),
    ("flyweight", "Flyweight"),
    ("womens-bantamweight", "Women's Bantamweight"),
    ("womens-flyweight", "Women's Flyweight"),
    ("womens-strawweight", "Women's Strawweight"),
];

/// Current view mode.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum View {
    Rankings,
    FighterDetail,
    Predictions,
    Optimize,
    BacktestConfig,
}

/// Current tab in the main view.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Tab {
    Rankings,
    Predictions,
    Optimize,
}

/// Optimization search method.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum OptimizationMethod {
    Grid,
    Random,
}

impl OptimizationMethod {
    pub fn name(&self) -> &'static str {
        match self {
            OptimizationMethod::Grid => "Grid Search",
            OptimizationMethod::Random => "Random Search",
        }
    }
}

/// Progress update from optimization task.
pub enum OptimizationProgress {
    Progress {
        current: usize,
        total: usize,
        config: EloConfig,
    },
    Complete {
        results: Vec<OptimizationResult>,
    },
    Error {
        message: String,
    },
}

/// Application state.
pub struct App {
    /// Database connection.
    pub database: Database,
    /// Current view.
    pub view: View,
    /// Current tab.
    pub tab: Tab,
    /// List of fighters for current view.
    pub fighters: Vec<Fighter>,
    /// Currently selected fighter index.
    pub selected_index: usize,
    /// Selected fighter for detail view.
    pub selected_fighter: Option<Fighter>,
    /// Fight history for selected fighter.
    pub fight_history: Vec<FightRecord>,
    /// Current weight class filter index.
    pub weight_class_index: usize,
    /// Scroll offset for rankings list.
    pub scroll_offset: usize,
    /// Whether the app should quit.
    pub should_quit: bool,
    /// Search query.
    pub search_query: String,
    /// Whether search mode is active.
    pub search_mode: bool,
    /// Upcoming event predictions.
    pub predictions: Vec<EventPrediction>,
    /// Selected event index in predictions view.
    pub selected_event_index: usize,
    /// Whether predictions are loading.
    pub predictions_loading: bool,
    /// Error message for predictions.
    pub predictions_error: Option<String>,

    // --- Active Config State ---
    /// The currently active Elo configuration (loaded from file or default).
    pub active_config: EloConfig,
    /// Whether a re-sync is needed (config changed but not applied).
    pub needs_resync: bool,

    // --- Optimizer State ---
    /// Cached fight data for optimization.
    pub fights_cache: Vec<Fight>,
    /// Optimization results.
    pub optimization_results: Vec<OptimizationResult>,
    /// Selected result index in the results table.
    pub selected_result_index: usize,
    /// Whether optimization is currently running.
    pub optimization_running: bool,
    /// Current optimization progress (current, total).
    pub optimization_progress: (usize, usize),
    /// Current config being tested (for display).
    pub optimization_current_config: Option<EloConfig>,
    /// Selected optimization method.
    pub optimization_method: OptimizationMethod,
    /// Number of random samples (for random search).
    pub random_samples: usize,
    /// Receiver for optimization progress updates.
    pub optimization_receiver: Option<mpsc::Receiver<OptimizationProgress>>,
    /// Status/error message from optimization.
    pub optimization_message: Option<(String, bool)>, // (message, is_success)
    /// Scroll offset for results table.
    pub results_scroll_offset: usize,

    // --- Custom Backtest State ---
    /// Custom backtest configuration.
    pub backtest_config: EloConfig,
    /// Which config field is selected (0=k, 1=finish, 2=title, 3=five_round).
    pub config_field_index: usize,
    /// Result from custom backtest.
    pub backtest_result: Option<BacktestResultDisplay>,
}

/// Display-friendly backtest result.
pub struct BacktestResultDisplay {
    pub log_loss: f64,
    pub brier_score: f64,
    pub accuracy: f64,
    pub fights_processed: usize,
}

/// A fight record with opponent name resolved.
pub struct FightRecord {
    pub opponent_name: String,
    pub is_win: bool,
    pub finish_method: String,
    pub date: String,
}

impl App {
    /// Creates a new App instance.
    pub async fn new(database: Database) -> anyhow::Result<Self> {
        let fighters = database.get_fighters_by_rating().await?;
        let active_config = EloConfig::load();

        Ok(Self {
            database,
            view: View::Rankings,
            tab: Tab::Rankings,
            fighters,
            selected_index: 0,
            selected_fighter: None,
            fight_history: Vec::new(),
            weight_class_index: 0,
            scroll_offset: 0,
            should_quit: false,
            search_query: String::new(),
            search_mode: false,
            predictions: Vec::new(),
            selected_event_index: 0,
            predictions_loading: false,
            predictions_error: None,

            // Active config
            active_config,
            needs_resync: false,

            // Optimizer state
            fights_cache: Vec::new(),
            optimization_results: Vec::new(),
            selected_result_index: 0,
            optimization_running: false,
            optimization_progress: (0, 0),
            optimization_current_config: None,
            optimization_method: OptimizationMethod::Grid,
            random_samples: 50,
            optimization_receiver: None,
            optimization_message: None,
            results_scroll_offset: 0,

            // Custom backtest state
            backtest_config: EloConfig::default(),
            config_field_index: 0,
            backtest_result: None,
        })
    }

    /// Loads fight data for optimization (cached).
    pub async fn load_fights_if_needed(&mut self) -> anyhow::Result<()> {
        if self.fights_cache.is_empty() {
            self.fights_cache = self.database.get_fights_order_by_date().await?;
        }
        Ok(())
    }

    /// Starts the optimization process in a background task.
    pub async fn start_optimization(&mut self) -> anyhow::Result<()> {
        if self.optimization_running {
            return Ok(());
        }

        // Load fights if needed
        self.load_fights_if_needed().await?;

        if self.fights_cache.is_empty() {
            self.optimization_message = Some((
                "No fight data available. Run sync first.".to_string(),
                false,
            ));
            return Ok(());
        }

        // Clear previous results
        self.optimization_results.clear();
        self.optimization_message = None;
        self.optimization_running = true;
        self.optimization_progress = (0, 0);
        self.selected_result_index = 0;
        self.results_scroll_offset = 0;

        // Create channel for progress updates
        let (tx, rx) = mpsc::channel::<OptimizationProgress>(100);
        self.optimization_receiver = Some(rx);

        // Clone data for the background task
        let fights = self.fights_cache.clone();
        let method = self.optimization_method;
        let random_samples = self.random_samples;

        // Spawn background task
        tokio::spawn(async move {
            run_optimization(tx, fights, method, random_samples).await;
        });

        Ok(())
    }

    /// Polls for optimization progress updates.
    pub fn poll_optimization(&mut self) {
        if let Some(ref mut rx) = self.optimization_receiver {
            // Process all available messages
            while let Ok(msg) = rx.try_recv() {
                match msg {
                    OptimizationProgress::Progress {
                        current,
                        total,
                        config,
                    } => {
                        self.optimization_progress = (current, total);
                        self.optimization_current_config = Some(config);
                    }
                    OptimizationProgress::Complete { results } => {
                        self.optimization_results = results;
                        self.optimization_running = false;
                        self.optimization_receiver = None;
                        self.optimization_current_config = None;
                        return;
                    }
                    OptimizationProgress::Error { message } => {
                        self.optimization_message = Some((message, false));
                        self.optimization_running = false;
                        self.optimization_receiver = None;
                        self.optimization_current_config = None;
                        return;
                    }
                }
            }
        }
    }

    /// Runs a custom backtest with the current config.
    pub async fn run_custom_backtest(&mut self) -> anyhow::Result<()> {
        self.load_fights_if_needed().await?;

        if self.fights_cache.is_empty() {
            return Ok(());
        }

        let mut backtester = Backtester::new();
        let result = backtester.run(&self.fights_cache, &self.backtest_config);

        self.backtest_result = Some(BacktestResultDisplay {
            log_loss: result.metrics.log_loss,
            brier_score: result.metrics.brier_score,
            accuracy: result.metrics.accuracy,
            fights_processed: result.fights_processed,
        });

        Ok(())
    }

    /// Applies the selected optimization result to the backtest config.
    pub fn apply_selected_result(&mut self) {
        if let Some(result) = self.optimization_results.get(self.selected_result_index) {
            self.backtest_config = result.config.clone();
        }
    }

    /// Saves the current backtest config as the active config.
    pub fn save_config(&mut self) {
        match self.backtest_config.save() {
            Ok(_) => {
                self.active_config = self.backtest_config.clone();
                self.needs_resync = true;
                self.optimization_message = Some((
                    "Config saved! Run 'sync' from CLI to apply to rankings.".to_string(),
                    true,
                ));
            }
            Err(e) => {
                self.optimization_message = Some((format!("Failed to save config: {}", e), false));
            }
        }
    }

    /// Saves the best optimization result as the active config.
    pub fn save_best_config(&mut self) {
        if let Some(result) = self.optimization_results.first() {
            self.backtest_config = result.config.clone();
            self.save_config();
        }
    }

    /// Adjusts the current backtest config field.
    pub fn adjust_config_field(&mut self, increase: bool) {
        let delta = if increase { 1.0 } else { -1.0 };

        match self.config_field_index {
            0 => {
                // K-factor: steps of 5
                self.backtest_config.k_factor =
                    (self.backtest_config.k_factor + delta * 5.0).clamp(10.0, 100.0);
            }
            1 => {
                // Finish multiplier: steps of 0.05
                self.backtest_config.finish_multiplier =
                    (self.backtest_config.finish_multiplier + delta * 0.05).clamp(1.0, 2.0);
            }
            2 => {
                // Title multiplier: steps of 0.05
                self.backtest_config.title_fight_multiplier =
                    (self.backtest_config.title_fight_multiplier + delta * 0.05).clamp(1.0, 2.0);
            }
            3 => {
                // Five-round multiplier: steps of 0.05
                self.backtest_config.five_round_multiplier =
                    (self.backtest_config.five_round_multiplier + delta * 0.05).clamp(1.0, 2.0);
            }
            _ => {}
        }
    }

    /// Loads predictions for upcoming events.
    pub async fn load_predictions(&mut self) {
        self.predictions_loading = true;
        self.predictions_error = None;

        match get_upcoming_predictions(&self.database).await {
            Ok(predictions) => {
                self.predictions = predictions;
                self.predictions_loading = false;
            }
            Err(e) => {
                self.predictions_error = Some(format!("Failed to load predictions: {}", e));
                self.predictions_loading = false;
            }
        }
    }

    /// Refreshes the fighter list based on current filter.
    pub async fn refresh_fighters(&mut self) -> anyhow::Result<()> {
        let (slug, _) = WEIGHT_CLASSES[self.weight_class_index];

        self.fighters = if slug == "all" {
            self.database.get_fighters_by_rating().await?
        } else {
            self.database.get_fighters_by_weight_class(slug).await?
        };

        // Apply search filter
        if !self.search_query.is_empty() {
            let query = self.search_query.to_lowercase();
            self.fighters
                .retain(|f| f.name.to_lowercase().contains(&query));
        }

        self.selected_index = 0;
        self.scroll_offset = 0;

        Ok(())
    }

    /// Loads fight history for the selected fighter.
    pub async fn load_fight_history(&mut self) -> anyhow::Result<()> {
        if let Some(ref fighter) = self.selected_fighter {
            let fights = self.database.get_fighter_fights(fighter.id).await?;
            self.fight_history.clear();

            for fight in fights {
                let is_win = fight.winner_id == fighter.id;
                let opponent_id = if is_win {
                    fight.loser_id
                } else {
                    fight.winner_id
                };

                let opponent_name = self
                    .database
                    .get_fighter_name(opponent_id)
                    .await
                    .unwrap_or_else(|_| "Unknown".to_string());

                let date = fight.date.date().to_string();

                self.fight_history.push(FightRecord {
                    opponent_name,
                    is_win,
                    finish_method: fight.finish_method.unwrap_or_default(),
                    date,
                });
            }
        }

        Ok(())
    }

    /// Handles key events.
    pub async fn handle_key(&mut self, key: KeyCode) -> anyhow::Result<()> {
        if self.search_mode {
            match key {
                KeyCode::Esc => {
                    self.search_mode = false;
                    self.search_query.clear();
                    self.refresh_fighters().await?;
                }
                KeyCode::Enter => {
                    self.search_mode = false;
                }
                KeyCode::Backspace => {
                    self.search_query.pop();
                    self.refresh_fighters().await?;
                }
                KeyCode::Char(c) => {
                    self.search_query.push(c);
                    self.refresh_fighters().await?;
                }
                _ => {}
            }
            return Ok(());
        }

        match self.view {
            View::Rankings => self.handle_rankings_key(key).await?,
            View::FighterDetail => self.handle_fighter_detail_key(key),
            View::Predictions => self.handle_predictions_key(key).await,
            View::Optimize => self.handle_optimize_key(key).await?,
            View::BacktestConfig => self.handle_backtest_config_key(key).await?,
        }

        Ok(())
    }

    async fn handle_rankings_key(&mut self, key: KeyCode) -> anyhow::Result<()> {
        match key {
            KeyCode::Char('q') => self.should_quit = true,
            KeyCode::Char('/') => self.search_mode = true,
            KeyCode::Tab => {
                self.tab = Tab::Predictions;
                self.view = View::Predictions;
                if self.predictions.is_empty() && !self.predictions_loading {
                    self.load_predictions().await;
                }
            }
            KeyCode::BackTab => {
                self.tab = Tab::Optimize;
                self.view = View::Optimize;
            }
            KeyCode::Up | KeyCode::Char('k') => {
                if self.selected_index > 0 {
                    self.selected_index -= 1;
                    if self.selected_index < self.scroll_offset {
                        self.scroll_offset = self.selected_index;
                    }
                }
            }
            KeyCode::Down | KeyCode::Char('j') => {
                if self.selected_index < self.fighters.len().saturating_sub(1) {
                    self.selected_index += 1;
                }
            }
            KeyCode::PageUp => {
                self.selected_index = self.selected_index.saturating_sub(20);
                self.scroll_offset = self.scroll_offset.saturating_sub(20);
            }
            KeyCode::PageDown => {
                self.selected_index =
                    (self.selected_index + 20).min(self.fighters.len().saturating_sub(1));
            }
            KeyCode::Home => {
                self.selected_index = 0;
                self.scroll_offset = 0;
            }
            KeyCode::End => {
                self.selected_index = self.fighters.len().saturating_sub(1);
            }
            KeyCode::Left | KeyCode::Char('h') => {
                if self.weight_class_index > 0 {
                    self.weight_class_index -= 1;
                    self.refresh_fighters().await?;
                }
            }
            KeyCode::Right | KeyCode::Char('l') => {
                if self.weight_class_index < WEIGHT_CLASSES.len() - 1 {
                    self.weight_class_index += 1;
                    self.refresh_fighters().await?;
                }
            }
            KeyCode::Enter => {
                if !self.fighters.is_empty() {
                    self.selected_fighter = Some(self.fighters[self.selected_index].clone());
                    self.load_fight_history().await?;
                    self.view = View::FighterDetail;
                }
            }
            _ => {}
        }
        Ok(())
    }

    fn handle_fighter_detail_key(&mut self, key: KeyCode) {
        match key {
            KeyCode::Char('q') | KeyCode::Esc | KeyCode::Backspace => {
                self.view = View::Rankings;
                self.selected_fighter = None;
                self.fight_history.clear();
            }
            _ => {}
        }
    }

    async fn handle_predictions_key(&mut self, key: KeyCode) {
        match key {
            KeyCode::Char('q') => self.should_quit = true,
            KeyCode::Tab => {
                self.tab = Tab::Optimize;
                self.view = View::Optimize;
            }
            KeyCode::BackTab => {
                self.tab = Tab::Rankings;
                self.view = View::Rankings;
            }
            KeyCode::Char('r') => {
                self.load_predictions().await;
            }
            KeyCode::Left | KeyCode::Char('h') => {
                if self.selected_event_index > 0 {
                    self.selected_event_index -= 1;
                }
            }
            KeyCode::Right | KeyCode::Char('l') => {
                if self.selected_event_index < self.predictions.len().saturating_sub(1) {
                    self.selected_event_index += 1;
                }
            }
            _ => {}
        }
    }

    async fn handle_optimize_key(&mut self, key: KeyCode) -> anyhow::Result<()> {
        // Don't allow most actions while optimization is running
        if self.optimization_running {
            if let KeyCode::Char('q') = key { self.should_quit = true }
            return Ok(());
        }

        match key {
            KeyCode::Char('q') => self.should_quit = true,
            KeyCode::Tab => {
                self.tab = Tab::Rankings;
                self.view = View::Rankings;
            }
            KeyCode::BackTab => {
                self.tab = Tab::Predictions;
                self.view = View::Predictions;
                if self.predictions.is_empty() && !self.predictions_loading {
                    self.load_predictions().await;
                }
            }
            KeyCode::Char('r') => {
                // Run optimization
                self.start_optimization().await?;
            }
            KeyCode::Char('m') | KeyCode::Left | KeyCode::Right => {
                // Toggle method
                self.optimization_method = match self.optimization_method {
                    OptimizationMethod::Grid => OptimizationMethod::Random,
                    OptimizationMethod::Random => OptimizationMethod::Grid,
                };
            }
            KeyCode::Char('+') | KeyCode::Char('=') => {
                // Increase random samples
                self.random_samples = (self.random_samples + 10).min(500);
            }
            KeyCode::Char('-') => {
                // Decrease random samples
                self.random_samples = self.random_samples.saturating_sub(10).max(10);
            }
            KeyCode::Up | KeyCode::Char('k') => {
                if self.selected_result_index > 0 {
                    self.selected_result_index -= 1;
                    if self.selected_result_index < self.results_scroll_offset {
                        self.results_scroll_offset = self.selected_result_index;
                    }
                }
            }
            KeyCode::Down | KeyCode::Char('j') => {
                if self.selected_result_index < self.optimization_results.len().saturating_sub(1) {
                    self.selected_result_index += 1;
                }
            }
            KeyCode::PageUp => {
                self.selected_result_index = self.selected_result_index.saturating_sub(20);
                self.results_scroll_offset = self.results_scroll_offset.saturating_sub(20);
            }
            KeyCode::PageDown => {
                self.selected_result_index = (self.selected_result_index + 20)
                    .min(self.optimization_results.len().saturating_sub(1));
            }
            KeyCode::Enter => {
                // Apply selected result and open backtest config
                if !self.optimization_results.is_empty() {
                    self.apply_selected_result();
                    self.view = View::BacktestConfig;
                }
            }
            KeyCode::Char('b') => {
                // Open custom backtest config
                self.view = View::BacktestConfig;
            }
            KeyCode::Char('e') => {
                // Export results to CSV
                if !self.optimization_results.is_empty() {
                    if let Err(e) = crate::engine::export_results_to_csv(
                        &self.optimization_results,
                        "optimization_results.csv",
                    ) {
                        self.optimization_message = Some((format!("Export failed: {}", e), false));
                    } else {
                        self.optimization_message = Some((
                            "Results exported to optimization_results.csv".to_string(),
                            true,
                        ));
                    }
                }
            }
            KeyCode::Char('a') => {
                // Apply best config
                if !self.optimization_results.is_empty() {
                    self.save_best_config();
                }
            }
            _ => {}
        }
        Ok(())
    }

    async fn handle_backtest_config_key(&mut self, key: KeyCode) -> anyhow::Result<()> {
        match key {
            KeyCode::Esc | KeyCode::Backspace => {
                self.view = View::Optimize;
                self.backtest_result = None;
            }
            KeyCode::Up | KeyCode::Char('k') => {
                if self.config_field_index > 0 {
                    self.config_field_index -= 1;
                }
            }
            KeyCode::Down | KeyCode::Char('j') => {
                if self.config_field_index < 3 {
                    self.config_field_index += 1;
                }
            }
            KeyCode::Left | KeyCode::Char('h') => {
                self.adjust_config_field(false);
            }
            KeyCode::Right | KeyCode::Char('l') => {
                self.adjust_config_field(true);
            }
            KeyCode::Enter => {
                self.run_custom_backtest().await?;
            }
            KeyCode::Char('s') => {
                // Save this config as active
                self.save_config();
            }
            _ => {}
        }
        Ok(())
    }
}

/// Runs optimization in a background task.
async fn run_optimization(
    tx: mpsc::Sender<OptimizationProgress>,
    fights: Vec<Fight>,
    method: OptimizationMethod,
    random_samples: usize,
) {
    let ranges = ParameterRanges::default();

    let tx_clone = tx.clone();
    let optimizer = Optimizer::with_ranges(ranges).with_progress(move |current, total, config| {
        let _ = tx_clone.try_send(OptimizationProgress::Progress {
            current,
            total,
            config: config.clone(),
        });
    });

    let result = match method {
        OptimizationMethod::Grid => {
            // Run grid search in blocking context
            tokio::task::spawn_blocking(move || optimizer.grid_search(&fights)).await
        }
        OptimizationMethod::Random => {
            let samples = random_samples;
            tokio::task::spawn_blocking(move || optimizer.random_search(&fights, samples, 42)).await
        }
    };

    match result {
        Ok((_, results)) => {
            let _ = tx.send(OptimizationProgress::Complete { results }).await;
        }
        Err(e) => {
            let _ = tx
                .send(OptimizationProgress::Error {
                    message: format!("Optimization failed: {}", e),
                })
                .await;
        }
    }
}

/// Runs the TUI application.
pub async fn run_app(database: Database) -> anyhow::Result<()> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create app state
    let mut app = App::new(database).await?;

    // Main loop
    loop {
        // Poll optimization progress
        app.poll_optimization();

        terminal.draw(|f| ui::draw(f, &app))?;

        if event::poll(std::time::Duration::from_millis(50))?
            && let Event::Key(key) = event::read()?
            && key.kind == KeyEventKind::Press
        {
            app.handle_key(key.code).await?;
        }

        if app.should_quit {
            break;
        }
    }

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    Ok(())
}
