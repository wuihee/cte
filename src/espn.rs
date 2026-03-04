//! # ESPN
//!
//! This module provides a small wrapper around ESPN's public UFC API. It is
//! responsible for sending requests and parsing responses into strongly typed
//! DTOs.

pub mod dto;

use anyhow::Result;
use reqwest::Client;
use time::OffsetDateTime;

use crate::espn::dto::{EventDto, EventsDto, FightCardDto};

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
    const EVENTS_API: &'static str = "https://sports.core.api.espn.com/v3/sports/mma/ufc/events";

    /// Base endpoint for retrieving data for a single fight card.
    ///
    /// # Example
    ///
    /// ```sh
    /// curl https://site.web.api.espn.com/apis/common/v3/sports/mma/ufc/fightcenter/600043333
    /// ```
    const FIGHT_CARD_API: &'static str =
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
        let url = format!("{}?season={year}", Espn::EVENTS_API);
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
        let url = format!("{}/{event_id}", Espn::FIGHT_CARD_API);
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

    /// Fetches upcoming UFC events (events with dates in the future).
    ///
    /// # Returns
    ///
    /// A vector of upcoming `EventDto` sorted by date, or an error on failure.
    pub async fn get_upcoming_events(&self) -> Result<Vec<EventDto>> {
        let now = OffsetDateTime::now_utc();
        let current_year = now.year();

        let mut upcoming = Vec::new();

        // Check current year and next year for upcoming events
        for year in [current_year, current_year + 1] {
            if let Ok(events) = self.get_all_events(year).await {
                for event in events.items {
                    if event.date > now {
                        upcoming.push(event);
                    }
                }
            }
        }

        // Sort by date
        upcoming.sort_by(|a, b| a.date.cmp(&b.date));

        Ok(upcoming)
    }
}

#[cfg(test)]
mod test {
    use crate::espn::Espn;

    #[tokio::test]
    async fn test_get_all_events() {
        let espn = Espn::new();
        let result = espn.get_all_events(2024).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_get_fight_card() {
        let espn = Espn::new();
        let result = espn.get_fight_card("600039753").await;
        assert!(result.is_ok());
    }
}
