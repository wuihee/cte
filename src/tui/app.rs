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

use crate::database::{Database, entities::Fighter};
use crate::engine::{EventPrediction, get_upcoming_predictions};

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
}

/// Current tab in the main view.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Tab {
    Rankings,
    Predictions,
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
        })
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
            View::Rankings => match key {
                KeyCode::Char('q') => self.should_quit = true,
                KeyCode::Char('/') => self.search_mode = true,
                KeyCode::Tab => {
                    self.tab = Tab::Predictions;
                    self.view = View::Predictions;
                    if self.predictions.is_empty() && !self.predictions_loading {
                        self.load_predictions().await;
                    }
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
            },
            View::FighterDetail => match key {
                KeyCode::Char('q') | KeyCode::Esc | KeyCode::Backspace => {
                    self.view = View::Rankings;
                    self.selected_fighter = None;
                    self.fight_history.clear();
                }
                _ => {}
            },
            View::Predictions => match key {
                KeyCode::Char('q') => self.should_quit = true,
                KeyCode::Tab => {
                    self.tab = Tab::Rankings;
                    self.view = View::Rankings;
                }
                KeyCode::Char('r') => {
                    // Reload predictions
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
            },
        }

        Ok(())
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
        terminal.draw(|f| ui::draw(f, &app))?;

        if event::poll(std::time::Duration::from_millis(100))?
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
