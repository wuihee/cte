//! # Engine
//!
//! This module contains the logic for updating fighter ratings.

pub mod calculator;
pub mod predictions;
pub mod sync;

pub use calculator::update_ratings;
pub use predictions::{get_upcoming_predictions, EventPrediction};
pub use sync::{sync_fight_data, SyncOptions};
