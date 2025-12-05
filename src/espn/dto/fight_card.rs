//! # Fight Card DTO
//!
//! DTO for the fight card endpoint.

use serde::Deserialize;

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
