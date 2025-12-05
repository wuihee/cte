//! # DTO
//!
//! This module defines the data transfer objects (DTOs) used to deserialize
//! the portion of the ESPN UFC event API that we care about.

pub mod events;
pub mod fight_card;

pub use events::EventsDto;
pub use fight_card::FightCardDto;
