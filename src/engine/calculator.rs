//! # Calculator
//!
//! This module calculates and updates fighter ratings.

use crate::database::{Database, entities::Fight};

pub async fn update_ratings(database: &Database) -> anyhow::Result<()> {
    let fights = sqlx::query_as!(
        Fight,
        r#"
        SELECT id, event_id, winner_id, loser_id
        FROM fights
        "#
    )
    .fetch_all(&database.pool)
    .await?;

    // Should I keep a `rating` and `max_rating` field in the `fighters` table?

    for fight in fights {}

    Ok(())
}
