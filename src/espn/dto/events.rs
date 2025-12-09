//! # Events DTO
//!
//! DTO for the endpoint to get all events.

use serde::Deserialize;

/// Represents a list of UFC events.
#[derive(Debug, Deserialize)]
pub struct EventsDto {
    pub items: Vec<EventDto>,
}

/// Represents a single UFC event.
#[derive(Debug, Deserialize)]
pub struct EventDto {
    pub id: String,
    pub date: String,
    pub name: String,
}
