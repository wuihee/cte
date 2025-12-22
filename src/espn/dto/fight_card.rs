//! # Fight Card DTO
//!
//! DTO for the fight card endpoint.

use serde::Deserialize;

/// Root UFC event payload.
#[derive(Debug, Deserialize)]
pub struct FightCardDto {
    pub cards: Option<CardsDto>,
}

/// Collection of card segments (main, prelims, early prelims).
#[derive(Debug, Deserialize)]
pub struct CardsDto {
    pub main: CardDto,
    pub prelims1: Option<CardDto>,
    pub prelims2: Option<CardDto>,
}

/// A single card segment such as Main Card.
#[derive(Debug, Deserialize)]
pub struct CardDto {
    pub competitions: Vec<CompetitionDto>,
}

/// A single fight entry.
#[derive(Debug, Deserialize)]
pub struct CompetitionDto {
    pub id: String,
    pub status: StatusDto,
    pub competitors: Vec<CompetitorDto>,
}

#[derive(Debug, Deserialize)]
pub struct StatusDto {
    pub clock: String,
    pub period: u32,
    pub result: ResultDto,
}

#[derive(Debug, Deserialize)]
pub struct ResultDto {
    /// kotko, sub, u-dec, s-dec, no contest, draw, etc.
    pub name: String,
}

/// Information about a competitor of a fight.
#[derive(Debug, Deserialize)]
pub struct CompetitorDto {
    pub athlete: AthleteDto,
    pub winner: bool,
    pub stats: Vec<StatsDto>,
}

/// Fighter information.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AthleteDto {
    pub id: String,
    pub full_name: String,
    pub weight_class: Option<WeightClassDto>,
}

#[derive(Debug, Deserialize)]
pub struct WeightClassDto {
    /// Lightweight, etc.
    pub slug: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StatsDto {
    /// Significant strikes, time in control, etc.
    pub name: String,
    pub value: f64,
    pub display_value: String,
}
