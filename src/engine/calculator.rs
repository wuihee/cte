//! # Calculator
//!
//! Calculates fighter ratings using an enhanced Elo system.
//!
//! Features:
//! - Dynamic K-factor based on fighter experience
//! - Finish bonuses (KO/TKO/Submissions)
//! - Early finish bonuses
//! - Expectation-weighted performance multiplier

use crate::database::Database;

/// Base K-factor for rating changes.
const BASE_K: f64 = 32.0;

/// Minimum K-factor for experienced fighters.
const MIN_K: f64 = 16.0;

/// Number of fights before K-factor stabilizes.
const K_STABILIZE_FIGHTS: f64 = 15.0;

/// Finish bonuses.
const KO_BONUS: f64 = 1.05;
const SUB_BONUS: f64 = 1.05;

/// Bonus for first-round finishes.
const FIRST_ROUND_BONUS: f64 = 1.02;

/// Maximum performance multiplier.
const MAX_MULTIPLIER: f64 = 1.15;

/// Updates all fighter ratings based on fight history.
pub async fn update_ratings(database: &Database) -> anyhow::Result<()> {
    let fights = database.get_fights_order_by_date().await?;

    for fight in fights {
        let winner = database.get_fighter(fight.winner_id).await?;
        let loser = database.get_fighter(fight.loser_id).await?;

        // --- Expected outcome ---
        let expected = expected_score(winner.rating, loser.rating);

        // --- Dynamic K factor ---
        let winner_fights = (winner.wins + winner.losses) as f64;
        let loser_fights = (loser.wins + loser.losses) as f64;

        let k = (dynamic_k_factor(winner_fights) + dynamic_k_factor(loser_fights)) / 2.0;

        // --- Performance multiplier ---
        let raw_multiplier =
            performance_multiplier(fight.finish_method.as_deref(), fight.fight_time);

        // Weight multiplier by expectation
        let multiplier = expectation_weighted_multiplier(raw_multiplier, expected);

        // --- Elo update ---
        let change = k * (1.0 - expected) * multiplier;

        let winner_rating = winner.rating + change;
        let loser_rating = (loser.rating - change).max(100.0);

        // --- Store rating history ---
        database
            .insert_rating(winner.id, fight.id, winner_rating)
            .await?;

        database
            .insert_rating(loser.id, fight.id, loser_rating)
            .await?;

        // --- Update fighters ---
        let winner_max = winner_rating.max(winner.max_rating);
        let weight_class = fight.weight_class.as_deref();

        database
            .update_fighter_after_fight(
                winner.id,
                winner_rating,
                winner_max,
                true,
                fight.finish_method.as_deref(),
                weight_class,
            )
            .await?;

        database
            .update_fighter_after_fight(
                loser.id,
                loser_rating,
                loser.max_rating,
                false,
                fight.finish_method.as_deref(),
                weight_class,
            )
            .await?;
    }

    Ok(())
}

/// Standard Elo expected score formula.
pub fn expected_score(rating_a: f64, rating_b: f64) -> f64 {
    1.0 / (1.0 + 10f64.powf((rating_b - rating_a) / 400.0))
}

/// Dynamic K-factor based on fighter experience.
pub fn dynamic_k_factor(fight_count: f64) -> f64 {
    let factor = (K_STABILIZE_FIGHTS - fight_count).max(0.0) / K_STABILIZE_FIGHTS;
    MIN_K + (BASE_K - MIN_K) * factor
}

/// Calculates base performance multiplier from fight outcome.
fn performance_multiplier(finish_method: Option<&str>, fight_time: i64) -> f64 {
    let mut multiplier = 1.0;

    if let Some(method) = finish_method {
        let method = method.to_ascii_uppercase();

        if method.contains("KO") || method.contains("TKO") {
            multiplier *= KO_BONUS;
        } else if method.contains("SUB") {
            multiplier *= SUB_BONUS;
        }
    }

    // First round finish (<= 5 minutes)
    if fight_time > 0 && fight_time <= 300 {
        multiplier *= FIRST_ROUND_BONUS;
    }

    multiplier.min(MAX_MULTIPLIER)
}

/// Reduces finish bonuses when the winner was already heavily favored.
///
/// Underdog finishes receive stronger bonuses.
fn expectation_weighted_multiplier(multiplier: f64, expected: f64) -> f64 {
    if multiplier <= 1.0 {
        return 1.0;
    }

    let bonus = multiplier - 1.0;

    // If expected win probability is high, reduce bonus
    let scaled_bonus = bonus * (1.0 - expected);

    (1.0 + scaled_bonus).min(MAX_MULTIPLIER)
}
