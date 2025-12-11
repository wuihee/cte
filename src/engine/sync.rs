//! # Sync
//!
//! This module is responsible for using ESPN's API to async fight data with
//! the database.

use crate::{
    database::Database,
    espn::{
        Espn,
        dto::{
            events::EventDto,
            fight_card::{CardDto, CompetitorDto},
        },
    },
};

/// Updates database will all fight data from ESPN.
///
/// # Arguments
///
/// - `database`: [`Database`] instance used to update the database..
///
/// # Returns
///
/// `Ok` if all data successfully updated, else `Err`.
pub async fn sync_fight_data(database: &Database) -> anyhow::Result<()> {
    let espn = Espn::new();

    for season in 1993..=2025 {
        let events = espn.get_all_events(season).await?;

        for event in events.items {
            println!("Syncing {} ({})", event.name, event.id);

            database
                .insert_event(&event.id, &event.name, &event.date)
                .await?;

            let fight_card = espn.get_fight_card(&event.id).await?;
            if let Some(cards) = fight_card.cards {
                insert_card(database, &event, &cards.main).await?;
                if let Some(card) = cards.prelims1 {
                    insert_card(database, &event, &card).await?;
                }
                if let Some(card) = cards.prelims2 {
                    insert_card(database, &event, &card).await?;
                }
            }
        }
    }

    Ok(())
}

async fn insert_card(database: &Database, event: &EventDto, card: &CardDto) -> anyhow::Result<()> {
    for competition in &card.competitions {
        let competitor_1 = &competition.competitors[0];
        let competitor_2 = &competition.competitors[1];

        insert_fighter(&database, competitor_1).await?;
        insert_fighter(&database, competitor_2).await?;

        if competitor_1.winner {
            database
                .insert_fight(
                    &competition.id,
                    &event.id,
                    &competitor_1.athlete.id,
                    &competitor_2.athlete.id,
                    &event.date,
                )
                .await?;
        } else {
            database
                .insert_fight(
                    &competition.id,
                    &event.id,
                    &competitor_2.athlete.id,
                    &competitor_1.athlete.id,
                    &event.date,
                )
                .await?;
        }
    }
    Ok(())
}

async fn insert_fighter(database: &Database, competitor: &CompetitorDto) -> anyhow::Result<()> {
    let id = &competitor.athlete.id;
    let name = &competitor.athlete.full_name;
    database.insert_fighter(id, name).await?;
    Ok(())
}
