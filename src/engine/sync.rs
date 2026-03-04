//! # Sync
//!
//! This module is responsible for using ESPN's API to sync fight data with
//! the database. Includes caching to avoid re-downloading already synced events.

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

/// Sync options for controlling sync behavior.
#[derive(Default)]
pub struct SyncOptions {
    /// If true, re-sync all events even if already cached.
    pub force: bool,
}

/// Updates database with all fight data from ESPN.
///
/// Uses a sync log to track which events have already been synced,
/// skipping them on subsequent runs unless `force` is true.
///
/// # Arguments
///
/// - `database`: [`Database`] instance used to update the database.
/// - `options`: Sync options controlling behavior.
///
/// # Returns
///
/// `Ok` if all data successfully updated, else `Err`.
pub async fn sync_fight_data(database: &Database, options: &SyncOptions) -> anyhow::Result<()> {
    let espn = Espn::new();

    // Show initial stats
    let (events_synced, fights_synced) = database.get_sync_stats().await?;
    if events_synced > 0 && !options.force {
        println!(
            "Cache: {} events, {} fights already synced",
            events_synced, fights_synced
        );
        println!("Use --force to re-sync all events\n");
    }

    // Clear cache if forcing
    if options.force && events_synced > 0 {
        println!("Force sync: clearing cache...");
        database.clear_sync_log().await?;
    }

    let mut total_new_events = 0;
    let mut total_new_fights = 0;
    let mut skipped_events = 0;

    for season in 1993..=2025 {
        let events = espn.get_all_events(season).await?;

        for event in events.items {
            // Check if already synced
            if !options.force && database.is_event_synced(&event.id).await? {
                skipped_events += 1;
                continue;
            }

            println!("Syncing {} ({})", event.name, event.id);

            database
                .insert_event(&event.id, &event.name, &event.date)
                .await?;

            let mut fights_count = 0;

            let fight_card = espn.get_fight_card(&event.id).await?;
            if let Some(cards) = fight_card.cards {
                fights_count += insert_card(database, &event, &cards.main).await?;
                if let Some(card) = cards.prelims1 {
                    fights_count += insert_card(database, &event, &card).await?;
                }
                if let Some(card) = cards.prelims2 {
                    fights_count += insert_card(database, &event, &card).await?;
                }
            }

            // Mark event as synced
            database
                .mark_event_synced(&event.id, &event.name, fights_count)
                .await?;

            total_new_events += 1;
            total_new_fights += fights_count;
        }
    }

    // Print summary
    println!();
    if skipped_events > 0 {
        println!("Skipped {} already synced events", skipped_events);
    }
    println!(
        "Synced {} new events with {} fights",
        total_new_events, total_new_fights
    );

    let (total_events, total_fights) = database.get_sync_stats().await?;
    println!(
        "Total in database: {} events, {} fights",
        total_events, total_fights
    );

    Ok(())
}

/// Inserts all fights from a card into the database.
///
/// For each competition in the card, this function:
/// 1. Inserts both fighters if they don't exist
/// 2. Inserts the fight record with winner/loser
/// 3. Inserts fight statistics for both competitors
///
/// # Arguments
///
/// - `database`: Database instance.
/// - `event`: The event DTO containing event information.
/// - `card`: The card DTO containing fight competitions.
///
/// # Returns
///
/// The number of fights inserted, or an error.
async fn insert_card(database: &Database, event: &EventDto, card: &CardDto) -> anyhow::Result<i32> {
    let mut count = 0;

    for competition in &card.competitions {
        if competition.competitors.len() < 2 {
            continue;
        }

        let competitor_1 = &competition.competitors[0];
        let competitor_2 = &competition.competitors[1];
        let status = &competition.status;
        let fight_time = status.period * 60 + (status.clock.parse::<f32>().unwrap_or(0.0) as u32);
        let weight_class = competitor_1
            .athlete
            .weight_class
            .as_ref()
            .map_or("", |w| &w.slug);
        let finish_method = &status.result.name;

        insert_fighter(database, competitor_1).await?;
        insert_fighter(database, competitor_2).await?;

        if competitor_1.winner {
            database
                .insert_fight(
                    &competition.id,
                    &event.id,
                    &competitor_1.athlete.id,
                    &competitor_2.athlete.id,
                    &event.date,
                    fight_time,
                    weight_class,
                    finish_method,
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
                    fight_time,
                    weight_class,
                    finish_method,
                )
                .await?;
        }

        // Insert fight stats for both competitors
        insert_fight_stats(database, &competition.id, competitor_1).await?;
        insert_fight_stats(database, &competition.id, competitor_2).await?;

        count += 1;
    }

    Ok(count)
}

/// Inserts a fighter into the database if they don't already exist.
///
/// # Arguments
///
/// - `database`: Database instance.
/// - `competitor`: The competitor DTO containing fighter information.
///
/// # Returns
///
/// `Ok` if successfully inserted, else `Err`.
async fn insert_fighter(database: &Database, competitor: &CompetitorDto) -> anyhow::Result<()> {
    let id = &competitor.athlete.id;
    let name = &competitor.athlete.full_name;
    database.insert_fighter(id, name).await?;
    Ok(())
}

/// Inserts fight statistics for a competitor into the database.
///
/// # Arguments
///
/// - `database`: Database instance.
/// - `fight_id`: The fight ID (ESPN competition ID).
/// - `competitor`: The competitor DTO containing stats.
///
/// # Returns
///
/// `Ok` if successfully inserted, else `Err`.
async fn insert_fight_stats(
    database: &Database,
    fight_id: &str,
    competitor: &CompetitorDto,
) -> anyhow::Result<()> {
    let mut knock_downs = 0;
    let mut total_strikes_hit = 0;
    let mut total_strikes_missed = 0;
    let mut sig_strikes = 0;
    let mut head_strikes = 0;
    let mut body_strikes = 0;
    let mut leg_strikes = 0;
    let mut time_in_control = 0;
    let mut takedowns_hit = 0;
    let mut takedowns_missed = 0;
    let mut submissions_hit = 0;
    let mut submissions_missed = 0;

    for stat in &competitor.stats {
        match stat.name.as_str() {
            "knockDowns" => knock_downs = stat.value as u32,
            "totalStrikes" => {
                total_strikes_hit = stat.value as u32;
                if let Some(missed) = stat.display_value.split("//").nth(1) {
                    total_strikes_missed = missed.trim().parse().unwrap_or(0);
                }
            }
            "sigStrikes" => sig_strikes = stat.value as u32,
            "headStrikes" => head_strikes = stat.value as u32,
            "bodyStrikes" => body_strikes = stat.value as u32,
            "legStrikes" => leg_strikes = stat.value as u32,
            "timeInControl" => time_in_control = stat.value as u32,
            "takedowns" => {
                takedowns_hit = stat.value as u32;
                if let Some(missed) = stat.display_value.split("//").nth(1) {
                    takedowns_missed = missed.trim().parse().unwrap_or(0);
                }
            }
            "submissions" => {
                submissions_hit = stat.value as u32;
                if let Some(missed) = stat.display_value.split("//").nth(1) {
                    submissions_missed = missed.trim().parse().unwrap_or(0);
                }
            }
            _ => {}
        }
    }

    database
        .insert_fight_stat(
            &competitor.athlete.id,
            fight_id,
            knock_downs,
            total_strikes_hit,
            total_strikes_missed,
            sig_strikes,
            head_strikes,
            body_strikes,
            leg_strikes,
            time_in_control,
            takedowns_hit,
            takedowns_missed,
            submissions_hit,
            submissions_missed,
        )
        .await?;

    Ok(())
}
