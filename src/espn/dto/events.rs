//! # Events DTO
//!
//! DTO for the endpoint to get all events.

use serde::Deserialize;
use time::OffsetDateTime;

/// Represents a list of UFC events.
#[derive(Debug, Deserialize)]
pub struct EventsDto {
    pub items: Vec<EventDto>,
}

/// Represents a single UFC event.
#[derive(Debug, Deserialize)]
pub struct EventDto {
    pub id: String,

    #[serde(with = "time::serde::iso8601")]
    pub date: OffsetDateTime,

    pub name: String,
}
