//! # Fight Card DTO
//!
//! DTO for the fight card endpoint.

use serde::Deserialize;

/// Represents the root of the parsed UFC event payload.
#[derive(Debug, Deserialize)]
pub struct FightCardDto {
    pub cards: Option<CardsDto>,
}

/// Contains the individual card segments of the event.
#[derive(Debug, Deserialize)]
pub struct CardsDto {
    /// Main card.
    pub main: CardDto,

    /// Prelims card.
    pub prelims1: Option<CardDto>,

    /// Early prelims card; only exists for the bigger events.
    pub prelims2: Option<CardDto>,
}

/// Represents a fight card segment such as the *Main Card*.
#[derive(Debug, Deserialize)]
pub struct CardDto {
    pub competitions: Vec<CompetitionDto>,
}

/// Represents a single fight on the card.
#[derive(Debug, Deserialize)]
pub struct CompetitionDto {
    pub id: String,
    pub competitors: Vec<CompetitorDto>,
}

/// Represents a single competitor in a fight.
#[derive(Debug, Deserialize)]
pub struct CompetitorDto {
    pub athlete: AthleteDto,
    pub winner: bool,
}

/// Represents the information for a fighter.
#[derive(Debug, Deserialize)]
pub struct AthleteDto {
    pub id: String,

    #[serde(rename = "fullName")]
    pub full_name: String,
}
