//! # ESPN
//!
//! This module provides a small wrapper around ESPN's public UFC API. It is
//! responsible for sending requests and parsing responses into strongly typed
//! DTOs.

pub mod dto;

use anyhow::Result;
use reqwest::Client;

use crate::espn::dto::{EventsDto, FightCardDto};

/// A lightweight client for fetching UFC-related data from ESPN's API.
pub struct Espn {
    /// HTTP client used for all API calls.
    client: Client,
}

impl Espn {
    /// Base endpoint for retrieving UFC events by year.
    ///
    /// # Example
    ///
    /// ```sh
    /// curl https://sports.core.api.espn.com/v3/sports/mma/ufc/events?season=2024
    /// ```
    const BASE_EVENTS_URL: &'static str =
        "https://sports.core.api.espn.com/v3/sports/mma/ufc/events";

    /// Base endpoint for retrieving data for a single fight card.
    ///
    /// # Example
    ///
    /// ```
    /// curl https://site.web.api.espn.com/apis/common/v3/sports/mma/ufc/fightcenter/600043333
    /// ```
    const BASE_FIGHT_CARD_URL: &'static str =
        "https://site.web.api.espn.com/apis/common/v3/sports/mma/ufc/fightcenter";

    /// Creates a new ESPN API client with a default `reqwest::Client`.
    pub fn new() -> Self {
        Espn {
            client: Client::new(),
        }
    }

    /// Fetches all UFC events for a given year.
    ///
    /// # Arguments
    ///
    /// - `year`: The season year you want to query, e.g. `2024`.
    ///
    /// # Returns
    ///
    /// A deserialized `EventsDto` or an error on failure.
    pub async fn get_all_events(&self, year: i32) -> Result<EventsDto> {
        let url = format!("{}?season={year}", Espn::BASE_EVENTS_URL);
        let events = self
            .client
            .get(&url)
            .send()
            .await?
            .error_for_status()?
            .json::<EventsDto>()
            .await?;
        Ok(events)
    }

    /// Fetches detailed information for a specific UFC fight card.
    ///
    /// # Arguments
    ///
    /// - `event_id`: The numeric ESPN event ID (e.g. `600043333`).
    ///
    /// # Returns
    ///
    /// A deserialized `FightCardDto` or an error on failure.
    pub async fn get_fight_card(&self, event_id: &str) -> Result<FightCardDto> {
        let url = format!("{}/{event_id}", Espn::BASE_FIGHT_CARD_URL);
        let fight_card = self
            .client
            .get(&url)
            .send()
            .await?
            .error_for_status()?
            .json::<FightCardDto>()
            .await?;
        Ok(fight_card)
    }
}
