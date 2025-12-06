//! # Domain
//!
//! This module defines the data model that is used by the rest of the app.
//! It's important that we decouple this from ESPN's DTO.

pub mod event;
pub mod fight;
pub mod fighter;
pub mod rating;

pub use event::Event;
pub use fight::Fight;
pub use fighter::Fighter;
pub use rating::Rating;
