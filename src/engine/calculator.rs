//! # Calculator
//!
//! This module calculates and updates fighter ratings using an enhanced Elo system.
//!
//! The enhanced system considers:
//! - Dynamic K-factor based on fighter experience
//! - Finish bonuses for KO/TKO and submissions
//! - Dominance factor based on fight statistics
//! - Round/time bonuses for quick finishes

use crate::database::Database;

/// Base K-factor for rating changes.
const BASE_K: f64 = 32.0;

/// Minimum K-factor for experienced fighters.
const MIN_K: f64 = 16.0;

/// Number of fights before K-factor stabilizes.
const K_STABILIZE_FIGHTS: f64 = 15.0;

/// Bonus multiplier for KO/TKO finishes.
const KO_BONUS: f64 = 1.25;

/// Bonus multiplier for submission finishes.
const SUB_BONUS: f64 = 1.20;

/// Maximum dominance bonus multiplier.
const MAX_DOMINANCE_BONUS: f64 = 1.3;

/// Bonus for first-round finishes.
const FIRST_ROUND_BONUS: f64 = 1.15;

/// Updates all fighter ratings based on fight history.
///
/// Processes fights chronologically and updates ratings using the enhanced
/// Elo system that considers finish type, dominance, and fighter experience.
pub async fn update_ratings(database: &Database) -> anyhow::Result<()> {
    let fights = database.get_fights_order_by_date().await?;

    for fight in fights {
        let winner = database.get_fighter(fight.winner_id).await?;
        let loser = database.get_fighter(fight.loser_id).await?;

        // Calculate expected scores
        let winner_expected = expected_score(winner.rating, loser.rating);
        let loser_expected = expected_score(loser.rating, winner.rating);

        // Get dynamic K-factors based on experience
        let winner_fights = winner.wins + winner.losses;
        let loser_fights = loser.wins + loser.losses;
        let winner_k = dynamic_k_factor(winner_fights as f64);
        let loser_k = dynamic_k_factor(loser_fights as f64);

        // Calculate performance multiplier based on fight details
        let finish_method = fight.finish_method.as_deref();
        let performance_multiplier = calculate_performance_multiplier(
            database,
            fight.id,
            fight.winner_id,
            fight.loser_id,
            finish_method,
            fight.fight_time,
        )
        .await;

        // Calculate rating changes
        let winner_change = winner_k * (1.0 - winner_expected) * performance_multiplier;
        let loser_change = loser_k * (0.0 - loser_expected);

        let winner_rating = winner.rating + winner_change;
        let loser_rating = (loser.rating + loser_change).max(100.0); // Floor at 100

        // Record rating history
        database
            .insert_rating(winner.id, fight.id, winner_rating)
            .await?;
        database
            .insert_rating(loser.id, fight.id, loser_rating)
            .await?;

        // Update fighter records
        let winner_max = winner_rating.max(winner.max_rating);
        let weight_class = fight.weight_class.as_deref();

        database
            .update_fighter_after_fight(
                winner.id,
                winner_rating,
                winner_max,
                true,
                finish_method,
                weight_class,
            )
            .await?;

        database
            .update_fighter_after_fight(
                loser.id,
                loser_rating,
                loser.max_rating,
                false,
                finish_method,
                weight_class,
            )
            .await?;
    }

    Ok(())
}

/// Calculates the expected score for fighter A against fighter B.
///
/// Uses the standard Elo formula: E_A = 1 / (1 + 10^((R_B - R_A) / 400))
pub fn expected_score(rating_a: f64, rating_b: f64) -> f64 {
    1.0 / (1.0 + 10f64.powf((rating_b - rating_a) / 400.0))
}

/// Calculates a dynamic K-factor based on fighter experience.
///
/// New fighters have higher K-factors (more volatile ratings) that decrease
/// as they accumulate fights, stabilizing after K_STABILIZE_FIGHTS.
pub fn dynamic_k_factor(fight_count: f64) -> f64 {
    let factor = (K_STABILIZE_FIGHTS - fight_count).max(0.0) / K_STABILIZE_FIGHTS;
    MIN_K + (BASE_K - MIN_K) * factor
}

/// Calculates the finish bonus multiplier based on how the fight ended.
#[cfg(test)]
pub fn finish_bonus(finish_method: Option<&str>) -> f64 {
    match finish_method {
        Some(method) => {
            let method_upper = method.to_uppercase();
            if method_upper.contains("KO") || method_upper.contains("TKO") {
                KO_BONUS
            } else if method_upper.contains("SUB") {
                SUB_BONUS
            } else {
                1.0
            }
        }
        None => 1.0,
    }
}

/// Returns the early finish bonus if applicable.
#[cfg(test)]
pub fn early_finish_bonus(fight_time: i64) -> f64 {
    if fight_time > 0 && fight_time <= 300 {
        FIRST_ROUND_BONUS
    } else {
        1.0
    }
}

/// Calculates a performance multiplier based on how the fight was won.
///
/// Considers:
/// - Finish type (KO/TKO, submission, decision)
/// - Round/time of finish (early finishes score higher)
/// - Strike and control dominance from fight statistics
async fn calculate_performance_multiplier(
    database: &Database,
    fight_id: i64,
    winner_id: i64,
    loser_id: i64,
    finish_method: Option<&str>,
    fight_time: i64,
) -> f64 {
    let mut multiplier = 1.0;

    // Apply finish type bonus
    if let Some(method) = finish_method {
        let method_upper = method.to_uppercase();
        if method_upper.contains("KO") || method_upper.contains("TKO") {
            multiplier *= KO_BONUS;
        } else if method_upper.contains("SUB") {
            multiplier *= SUB_BONUS;
        }
    }

    // Apply early finish bonus (first round = under 5 minutes)
    if fight_time > 0 && fight_time <= 300 {
        multiplier *= FIRST_ROUND_BONUS;
    }

    // Calculate dominance bonus from fight statistics
    let dominance = calculate_dominance_factor(database, fight_id, winner_id, loser_id).await;
    multiplier *= 1.0 + (dominance * (MAX_DOMINANCE_BONUS - 1.0));

    multiplier
}

/// Calculates a dominance factor (0.0 to 1.0) based on fight statistics.
///
/// Considers:
/// - Knockdown differential
/// - Significant strike differential
/// - Control time differential
/// - Takedown success
async fn calculate_dominance_factor(
    database: &Database,
    fight_id: i64,
    winner_id: i64,
    loser_id: i64,
) -> f64 {
    let winner_stats = database
        .get_fight_stats(winner_id, fight_id)
        .await
        .ok()
        .flatten();
    let loser_stats = database
        .get_fight_stats(loser_id, fight_id)
        .await
        .ok()
        .flatten();

    match (winner_stats, loser_stats) {
        (Some(w), Some(l)) => {
            let mut factors: Vec<f64> = Vec::new();

            // Knockdown factor (knockdowns are very significant)
            let w_kd = w.knock_downs.unwrap_or(0) as f64;
            let l_kd = l.knock_downs.unwrap_or(0) as f64;
            if w_kd > 0.0 || l_kd > 0.0 {
                let kd_factor = (w_kd - l_kd) / (w_kd + l_kd + 1.0);
                factors.push(kd_factor.clamp(-1.0, 1.0) * 0.3); // Weight: 30%
            }

            // Significant strikes differential
            let w_sig = w.sig_strikes.unwrap_or(0) as f64;
            let l_sig = l.sig_strikes.unwrap_or(0) as f64;
            if w_sig > 0.0 || l_sig > 0.0 {
                let sig_factor = (w_sig - l_sig) / (w_sig + l_sig);
                factors.push(sig_factor.clamp(-1.0, 1.0) * 0.25); // Weight: 25%
            }

            // Control time differential
            let w_ctrl = w.time_in_control.unwrap_or(0) as f64;
            let l_ctrl = l.time_in_control.unwrap_or(0) as f64;
            if w_ctrl > 0.0 || l_ctrl > 0.0 {
                let ctrl_factor = (w_ctrl - l_ctrl) / (w_ctrl + l_ctrl + 1.0);
                factors.push(ctrl_factor.clamp(-1.0, 1.0) * 0.25); // Weight: 25%
            }

            // Takedown success
            let w_td = w.takedowns_hit.unwrap_or(0) as f64;
            let l_td = l.takedowns_hit.unwrap_or(0) as f64;
            if w_td > 0.0 || l_td > 0.0 {
                let td_factor = (w_td - l_td) / (w_td + l_td + 1.0);
                factors.push(td_factor.clamp(-1.0, 1.0) * 0.2); // Weight: 20%
            }

            // Average all factors and normalize to 0.0-1.0 range
            if factors.is_empty() {
                0.0
            } else {
                let sum: f64 = factors.iter().sum();
                ((sum / factors.len() as f64) + 1.0) / 2.0
            }
        }
        _ => 0.0, // No stats available
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Helper to check floating point equality within tolerance.
    fn approx_eq(a: f64, b: f64, tolerance: f64) -> bool {
        (a - b).abs() < tolerance
    }

    fn test_expected_score_equal_ratings() {
        // Equal ratings should give 50% expected score
        let score = expected_score(1000.0, 1000.0);
        assert!(approx_eq(score, 0.5, 0.001));
    }

    #[test]
    fn test_expected_score_higher_rating_favored() {
        // Higher rated fighter should be favored
        let score = expected_score(1200.0, 1000.0);
        assert!(score > 0.5);
        assert!(score < 1.0);
    }

    #[test]
    fn test_expected_score_lower_rating_underdog() {
        // Lower rated fighter should be underdog
        let score = expected_score(1000.0, 1200.0);
        assert!(score < 0.5);
        assert!(score > 0.0);
    }

    #[test]
    fn test_expected_score_symmetry() {
        // Probabilities should sum to 1
        let score_a = expected_score(1200.0, 1000.0);
        let score_b = expected_score(1000.0, 1200.0);
        assert!(approx_eq(score_a + score_b, 1.0, 0.001));
    }

    #[test]
    fn test_expected_score_400_point_difference() {
        // 400 point difference should give ~90.9% to higher rated
        let score = expected_score(1400.0, 1000.0);
        assert!(approx_eq(score, 0.909, 0.01));
    }

    #[test]
    fn test_expected_score_large_difference() {
        // Very large difference should approach but not reach 1.0
        let score = expected_score(2000.0, 1000.0);
        assert!(score > 0.99);
        assert!(score < 1.0);
    }

    fn test_k_factor_new_fighter() {
        // New fighter (0 fights) should have max K-factor
        let k = dynamic_k_factor(0.0);
        assert!(approx_eq(k, BASE_K, 0.001));
    }

    #[test]
    fn test_k_factor_experienced_fighter() {
        // Experienced fighter (15+ fights) should have min K-factor
        let k = dynamic_k_factor(15.0);
        assert!(approx_eq(k, MIN_K, 0.001));
    }

    #[test]
    fn test_k_factor_very_experienced_fighter() {
        // Very experienced fighter should still have min K-factor
        let k = dynamic_k_factor(50.0);
        assert!(approx_eq(k, MIN_K, 0.001));
    }

    #[test]
    fn test_k_factor_mid_career() {
        // Mid-career fighter should have intermediate K-factor
        let k = dynamic_k_factor(7.5);
        let expected = (BASE_K + MIN_K) / 2.0; // Should be halfway
        assert!(approx_eq(k, expected, 0.001));
    }

    #[test]
    fn test_k_factor_decreases_with_experience() {
        // K-factor should decrease as fights increase
        let k_0 = dynamic_k_factor(0.0);
        let k_5 = dynamic_k_factor(5.0);
        let k_10 = dynamic_k_factor(10.0);
        let k_15 = dynamic_k_factor(15.0);

        assert!(k_0 > k_5);
        assert!(k_5 > k_10);
        assert!(k_10 > k_15);
    }

    fn test_finish_bonus_ko() {
        assert!(approx_eq(finish_bonus(Some("KO")), KO_BONUS, 0.001));
        assert!(approx_eq(finish_bonus(Some("ko")), KO_BONUS, 0.001));
        assert!(approx_eq(finish_bonus(Some("KO/TKO")), KO_BONUS, 0.001));
    }

    #[test]
    fn test_finish_bonus_tko() {
        assert!(approx_eq(finish_bonus(Some("TKO")), KO_BONUS, 0.001));
        assert!(approx_eq(finish_bonus(Some("tko")), KO_BONUS, 0.001));
    }

    #[test]
    fn test_finish_bonus_submission() {
        assert!(approx_eq(finish_bonus(Some("SUB")), SUB_BONUS, 0.001));
        assert!(approx_eq(
            finish_bonus(Some("Submission")),
            SUB_BONUS,
            0.001
        ));
        assert!(approx_eq(finish_bonus(Some("sub")), SUB_BONUS, 0.001));
    }

    #[test]
    fn test_finish_bonus_decision() {
        assert!(approx_eq(finish_bonus(Some("Decision")), 1.0, 0.001));
        assert!(approx_eq(finish_bonus(Some("U-DEC")), 1.0, 0.001));
        assert!(approx_eq(finish_bonus(Some("S-DEC")), 1.0, 0.001));
    }

    #[test]
    fn test_finish_bonus_none() {
        assert!(approx_eq(finish_bonus(None), 1.0, 0.001));
    }

    fn test_early_finish_bonus_first_round() {
        // Under 5 minutes (300 seconds) should get bonus
        assert!(approx_eq(early_finish_bonus(60), FIRST_ROUND_BONUS, 0.001));
        assert!(approx_eq(early_finish_bonus(180), FIRST_ROUND_BONUS, 0.001));
        assert!(approx_eq(early_finish_bonus(300), FIRST_ROUND_BONUS, 0.001));
    }

    #[test]
    fn test_early_finish_bonus_later_rounds() {
        // Over 5 minutes should not get bonus
        assert!(approx_eq(early_finish_bonus(301), 1.0, 0.001));
        assert!(approx_eq(early_finish_bonus(600), 1.0, 0.001));
        assert!(approx_eq(early_finish_bonus(900), 1.0, 0.001));
    }

    #[test]
    fn test_early_finish_bonus_zero_time() {
        // Zero time (missing data) should not get bonus
        assert!(approx_eq(early_finish_bonus(0), 1.0, 0.001));
    }

    #[test]
    fn test_early_finish_bonus_negative_time() {
        // Negative time (invalid) should not get bonus
        assert!(approx_eq(early_finish_bonus(-100), 1.0, 0.001));
    }

    fn test_upset_victory_rating_change() {
        // Underdog wins - should gain more points
        let underdog_rating = 900.0;
        let favorite_rating = 1100.0;
        let k = 32.0;

        let expected = expected_score(underdog_rating, favorite_rating);
        let change = k * (1.0 - expected);

        // Underdog expected to lose, so big gain when winning
        assert!(change > k * 0.5); // Should gain more than half of K
    }

    #[test]
    fn test_expected_victory_rating_change() {
        // Favorite wins - should gain fewer points
        let favorite_rating = 1100.0;
        let underdog_rating = 900.0;
        let k = 32.0;

        let expected = expected_score(favorite_rating, underdog_rating);
        let change = k * (1.0 - expected);

        // Favorite expected to win, so small gain
        assert!(change < k * 0.5); // Should gain less than half of K
    }

    #[test]
    fn test_rating_floor() {
        // Ratings should not go below 100
        let initial = 150.0;
        let k = 32.0;
        let expected = expected_score(initial, 1500.0); // Very unfavorable matchup

        let change = k * (0.0 - expected); // Loss
        let new_rating = (initial + change).max(100.0);

        assert!(new_rating >= 100.0);
    }
}
