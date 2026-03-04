//! # Calculator
//!
//! Calculates fighter ratings using an enhanced Elo system.
//!
//! Features:
//! - Configurable K-factor and multipliers via EloConfig
//! - Dynamic K-factor based on fighter experience
//! - Finish bonuses (KO/TKO/Submissions)
//! - Title fight and five-round fight multipliers
//! - Expectation-weighted performance multiplier

use crate::database::Database;
use crate::engine::config::EloConfig;

/// Minimum K-factor for experienced fighters (used in dynamic K calculation).
const MIN_K_RATIO: f64 = 0.5;

/// Number of fights before K-factor stabilizes.
const K_STABILIZE_FIGHTS: f64 = 15.0;

/// Maximum performance multiplier.
const MAX_MULTIPLIER: f64 = 1.25;

/// Updates all fighter ratings based on fight history using the given configuration.
///
/// # Arguments
///
/// * `database` - Database connection for retrieving fights and updating ratings.
/// * `config` - Elo configuration parameters.
pub async fn update_ratings(database: &Database, config: &EloConfig) -> anyhow::Result<()> {
    let fights = database.get_fights_order_by_date().await?;

    for fight in fights {
        let winner = database.get_fighter(fight.winner_id).await?;
        let loser = database.get_fighter(fight.loser_id).await?;

        // --- Expected outcome ---
        let expected = expected_score(winner.rating, loser.rating);

        // --- Dynamic K factor ---
        let winner_fights = (winner.wins + winner.losses) as f64;
        let loser_fights = (loser.wins + loser.losses) as f64;

        let k = (dynamic_k_factor(config.k_factor, winner_fights)
            + dynamic_k_factor(config.k_factor, loser_fights))
            / 2.0;

        // --- Performance multiplier ---
        let raw_multiplier = performance_multiplier(
            config,
            fight.finish_method.as_deref(),
            fight.weight_class.as_deref(),
            fight.fight_time,
        );

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

/// Updates all fighter ratings using default configuration.
///
/// This is a convenience function that uses `EloConfig::default()`.
#[allow(dead_code)]
pub async fn update_ratings_default(database: &Database) -> anyhow::Result<()> {
    update_ratings(database, &EloConfig::default()).await
}

/// Standard Elo expected score formula.
///
/// Returns the probability that fighter A wins against fighter B.
pub fn expected_score(rating_a: f64, rating_b: f64) -> f64 {
    1.0 / (1.0 + 10f64.powf((rating_b - rating_a) / 400.0))
}

/// Dynamic K-factor based on fighter experience.
///
/// New fighters have higher K (more volatile ratings).
/// Veterans have lower K (more stable ratings).
pub fn dynamic_k_factor(base_k: f64, fight_count: f64) -> f64 {
    let min_k = base_k * MIN_K_RATIO;
    let factor = (K_STABILIZE_FIGHTS - fight_count).max(0.0) / K_STABILIZE_FIGHTS;
    min_k + (base_k - min_k) * factor
}

/// Calculates performance multiplier from fight outcome and context.
///
/// Applies finish, title fight, and five-round fight multipliers.
fn performance_multiplier(
    config: &EloConfig,
    finish_method: Option<&str>,
    weight_class: Option<&str>,
    fight_time: i64,
) -> f64 {
    let mut multiplier = 1.0;

    // Check for finish (KO/TKO/Submission)
    if let Some(method) = finish_method {
        let method_upper = method.to_ascii_uppercase();
        if method_upper.contains("KO")
            || method_upper.contains("TKO")
            || method_upper.contains("SUB")
        {
            multiplier *= config.finish_multiplier;
        }
    }

    // Check for title fight (heuristic: weight class contains "title" or "championship")
    if let Some(wc) = weight_class {
        let wc_upper = wc.to_ascii_uppercase();
        if wc_upper.contains("TITLE") || wc_upper.contains("CHAMPIONSHIP") {
            multiplier *= config.title_fight_multiplier;
        }
    }

    // Check for five-round fight (fight time > 15 minutes indicates scheduled for 5 rounds)
    // This is a heuristic - fights that go past round 3 (15 min) are likely 5-rounders
    if fight_time > 900 {
        multiplier *= config.five_round_multiplier;
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_expected_score_equal_ratings() {
        let score = expected_score(1000.0, 1000.0);
        assert!((score - 0.5).abs() < 0.001);
    }

    #[test]
    fn test_expected_score_higher_rating() {
        let score = expected_score(1200.0, 1000.0);
        assert!(score > 0.5);
        assert!(score < 1.0);
    }

    #[test]
    fn test_dynamic_k_factor() {
        let base_k = 32.0;
        // New fighter should have higher K
        let new_k = dynamic_k_factor(base_k, 0.0);
        assert_eq!(new_k, base_k);

        // Veteran should have lower K
        let vet_k = dynamic_k_factor(base_k, 20.0);
        assert_eq!(vet_k, base_k * MIN_K_RATIO);
    }
}
