//! # Engine
//!
//! This module contains the logic for updating fighter ratings,
//! backtesting configurations, and optimizing parameters.

pub mod backtest;
pub mod calculator;
pub mod config;
pub mod metrics;
pub mod optimizer;
pub mod predictions;
pub mod sync;

pub use backtest::Backtester;
pub use calculator::update_ratings;
pub use config::{EloConfig, ParameterRanges};
pub use optimizer::{OptimizationResult, Optimizer, export_results_to_csv, print_top_results};
pub use predictions::{EventPrediction, get_upcoming_predictions};
pub use sync::{SyncOptions, sync_fight_data};
