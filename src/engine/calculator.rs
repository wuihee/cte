//! # Calculator
//!
//! This module calculates and updates fighter ratings.

use crate::database::Database;

pub async fn update_ratings(database: &Database) -> anyhow::Result<()> {
    let k = 30.0;
    let fights = database.get_fights_order_by_date().await?;

    for fight in fights {
        let winner = database.get_fighter(fight.winner_id).await?;
        let loser = database.get_fighter(fight.loser_id).await?;

        let winner_expected = get_expected_rating(winner.rating, loser.rating);
        let loser_expected = get_expected_rating(loser.rating, winner.rating);

        let winner_rating = winner.rating + k * (1.0 - winner_expected);
        let loser_rating = loser.rating + k * (0.0 - loser_expected);

        database
            .insert_rating(winner.id, fight.id, winner_rating)
            .await?;
        database
            .insert_rating(loser.id, fight.id, loser_rating)
            .await?;

        let winner_max_rating = if winner_rating > winner.max_rating {
            winner_rating
        } else {
            winner.max_rating
        };

        database
            .update_figher_rating(winner.id, winner_rating, winner_max_rating)
            .await?;
        database
            .update_figher_rating(loser.id, loser_rating, loser.max_rating)
            .await?;
    }

    Ok(())
}

pub fn get_expected_rating(rating_a: f64, rating_b: f64) -> f64 {
    1.0 / (1.0 + 10f64.powf((rating_b - rating_a) / 400.0))
}
