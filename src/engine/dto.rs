//! # DTO
//!
//! This module defines the data transfer objects (DTOs) used to deserialize
//! the portion of the ESPN UFC event API that we care about.

use serde::Deserialize;

/// Represents a list of UFC events.
#[derive(Debug, Deserialize)]
pub struct EventsDto {
    pub count: usize,
    pub items: Vec<EventDto>,
}

/// Represents a single UFC event.
#[derive(Debug, Deserialize)]
pub struct EventDto {
    pub id: String,
    pub date: String,
    pub name: String,
}

/// Represents the root of the parsed UFC event payload.
#[derive(Debug, Deserialize)]
pub struct FightCardDto {
    pub cards: CardsDto,
}

/// Contains the individual card segments of the event.
///
/// ESPN exposes multiple segments (e.g., *early prelims*, *prelims*,
/// *main card*).
#[derive(Debug, Deserialize)]
pub struct CardsDto {
    pub main: CardDto,
}

/// Represents a fight card segment such as the *Main Card*.
#[derive(Debug, Deserialize)]
pub struct CardDto {
    pub competitions: Vec<CompetitionDto>,
}

/// Represents a single fight on the card.
#[derive(Debug, Deserialize)]
pub struct CompetitionDto {
    pub competitors: Vec<CompetitorDto>,
}

/// Represents a single competitor in a fight.
#[derive(Debug, Deserialize)]
pub struct CompetitorDto {
    pub athlete: AthleteDto,
    pub winner: Option<bool>,
}

/// Represents the information for a fighter.
#[derive(Debug, Deserialize)]
pub struct AthleteDto {
    #[serde(rename = "fullName")]
    pub full_name: String,
}
