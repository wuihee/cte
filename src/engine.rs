//! # Engine
//!
//! This module contains the logic for updating fighter ratings.

pub mod calculator;
pub mod predictions;
pub mod sync;

pub use calculator::update_ratings;
pub use predictions::{EventPrediction, get_upcoming_predictions};
pub use sync::{SyncOptions, sync_fight_data};
