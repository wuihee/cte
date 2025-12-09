//! # Engine
//!
//! This module contains the logic for updating fighter ratings.

pub mod calculator;
pub mod sync;

pub use calculator::update_ratings;
pub use sync::sync_fight_data;
