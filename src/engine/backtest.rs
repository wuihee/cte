//! # Backtesting Engine
//!
//! Runs historical simulations to evaluate Elo configuration performance.
//!
//! The backtester processes fights chronologically, making predictions
//! BEFORE updating ratings, which is critical for proper evaluation.

use std::collections::HashMap;

use crate::database::entities::Fight;
use crate::engine::calculator::{dynamic_k_factor, expected_score};
use crate::engine::config::EloConfig;
use crate::engine::metrics::{EvaluationMetrics, PredictionRecord, calculate_metrics};

/// Minimum K-factor ratio (same as calculator).
#[allow(dead_code)]
const MIN_K_RATIO: f64 = 0.5;

/// Maximum performance multiplier.
const MAX_MULTIPLIER: f64 = 1.25;

/// Default starting rating for fighters.
const DEFAULT_RATING: f64 = 1000.0;

/// Minimum rating floor.
const MIN_RATING: f64 = 100.0;

/// In-memory fighter state for backtesting.
#[derive(Debug, Clone)]
struct FighterState {
    rating: f64,
    wins: i64,
    losses: i64,
}

impl Default for FighterState {
    fn default() -> Self {
        Self {
            rating: DEFAULT_RATING,
            wins: 0,
            losses: 0,
        }
    }
}

/// Result of a single backtest run.
#[derive(Debug, Clone)]
pub struct BacktestResult {
    /// Evaluation metrics for this backtest.
    pub metrics: EvaluationMetrics,

    /// All prediction records (for detailed analysis).
    #[allow(dead_code)]
    pub predictions: Vec<PredictionRecord>,

    /// Number of fights processed.
    pub fights_processed: usize,
}

impl BacktestResult {
    /// Creates an empty backtest result.
    #[allow(dead_code)]
    pub fn empty() -> Self {
        Self {
            metrics: EvaluationMetrics::default(),
            predictions: Vec::new(),
            fights_processed: 0,
        }
    }
}

/// Backtesting engine that simulates the Elo system on historical data.
///
/// This engine maintains fighter ratings in memory (not in the database)
/// to allow fast repeated backtests with different configurations.
pub struct Backtester {
    /// Fighter states indexed by fighter ID.
    fighters: HashMap<i64, FighterState>,
}

impl Backtester {
    /// Creates a new backtester with empty fighter state.
    pub fn new() -> Self {
        Self {
            fighters: HashMap::new(),
        }
    }

    /// Resets all fighter ratings to default values.
    pub fn reset(&mut self) {
        self.fighters.clear();
    }

    /// Gets or creates a fighter state.
    fn get_fighter(&mut self, id: i64) -> &FighterState {
        self.fighters.entry(id).or_default()
    }

    /// Gets mutable fighter state.
    fn get_fighter_mut(&mut self, id: i64) -> &mut FighterState {
        self.fighters.entry(id).or_default()
    }

    /// Runs a backtest on the given fights with the specified configuration.
    ///
    /// This method:
    /// 1. Resets all fighter ratings
    /// 2. Processes fights chronologically
    /// 3. For each fight: predicts outcome BEFORE updating ratings
    /// 4. Returns evaluation metrics
    ///
    /// # Arguments
    ///
    /// * `fights` - Fights to process, should be sorted by date ascending.
    /// * `config` - Elo configuration to use.
    ///
    /// # Returns
    ///
    /// `BacktestResult` containing metrics and predictions.
    pub fn run(&mut self, fights: &[Fight], config: &EloConfig) -> BacktestResult {
        // Reset state for fresh backtest
        self.reset();

        let mut predictions = Vec::with_capacity(fights.len());

        for fight in fights {
            // Get current ratings BEFORE update
            let winner_state = self.get_fighter(fight.winner_id).clone();
            let loser_state = self.get_fighter(fight.loser_id).clone();

            // Calculate win probability for the winner
            let win_prob = expected_score(winner_state.rating, loser_state.rating);

            // Record prediction (from winner's perspective, actual = 1.0)
            predictions.push(PredictionRecord::winner_prediction(win_prob));

            // Now update ratings
            self.update_fight(fight, config, &winner_state, &loser_state);
        }

        // Calculate metrics
        let metrics = calculate_metrics(&predictions);

        BacktestResult {
            metrics,
            predictions,
            fights_processed: fights.len(),
        }
    }

    /// Updates ratings after a fight.
    fn update_fight(
        &mut self,
        fight: &Fight,
        config: &EloConfig,
        winner_state: &FighterState,
        loser_state: &FighterState,
    ) {
        let expected = expected_score(winner_state.rating, loser_state.rating);

        // Dynamic K factor
        let winner_fights = (winner_state.wins + winner_state.losses) as f64;
        let loser_fights = (loser_state.wins + loser_state.losses) as f64;

        let k = (dynamic_k_factor(config.k_factor, winner_fights)
            + dynamic_k_factor(config.k_factor, loser_fights))
            / 2.0;

        // Performance multiplier
        let raw_multiplier = self.performance_multiplier(
            config,
            fight.finish_method.as_deref(),
            fight.weight_class.as_deref(),
            fight.fight_time,
        );

        let multiplier = expectation_weighted_multiplier(raw_multiplier, expected);

        // Calculate rating change
        let change = k * (1.0 - expected) * multiplier;

        // Update winner
        let winner = self.get_fighter_mut(fight.winner_id);
        winner.rating += change;
        winner.wins += 1;

        // Update loser
        let loser = self.get_fighter_mut(fight.loser_id);
        loser.rating = (loser.rating - change).max(MIN_RATING);
        loser.losses += 1;
    }

    /// Calculates performance multiplier based on fight context.
    fn performance_multiplier(
        &self,
        config: &EloConfig,
        finish_method: Option<&str>,
        weight_class: Option<&str>,
        fight_time: i64,
    ) -> f64 {
        let mut multiplier = 1.0;

        // Finish bonus
        if let Some(method) = finish_method {
            let method_upper = method.to_ascii_uppercase();
            if method_upper.contains("KO")
                || method_upper.contains("TKO")
                || method_upper.contains("SUB")
            {
                multiplier *= config.finish_multiplier;
            }
        }

        // Title fight bonus
        if let Some(wc) = weight_class {
            let wc_upper = wc.to_ascii_uppercase();
            if wc_upper.contains("TITLE") || wc_upper.contains("CHAMPIONSHIP") {
                multiplier *= config.title_fight_multiplier;
            }
        }

        // Five-round fight bonus (fights > 15 min are likely 5-rounders)
        if fight_time > 900 {
            multiplier *= config.five_round_multiplier;
        }

        multiplier.min(MAX_MULTIPLIER)
    }
}

impl Default for Backtester {
    fn default() -> Self {
        Self::new()
    }
}

/// Reduces bonuses when the winner was heavily favored.
fn expectation_weighted_multiplier(multiplier: f64, expected: f64) -> f64 {
    if multiplier <= 1.0 {
        return 1.0;
    }

    let bonus = multiplier - 1.0;
    let scaled_bonus = bonus * (1.0 - expected);

    (1.0 + scaled_bonus).min(MAX_MULTIPLIER)
}

#[cfg(test)]
mod tests {
    use super::*;
    use time::OffsetDateTime;

    fn create_test_fight(winner_id: i64, loser_id: i64) -> Fight {
        Fight {
            id: 1,
            event_id: 1,
            winner_id,
            loser_id,
            date: OffsetDateTime::now_utc(),
            fight_time: 300,
            weight_class: Some("lightweight".to_string()),
            finish_method: Some("KO".to_string()),
        }
    }

    #[test]
    fn test_backtester_reset() {
        let mut bt = Backtester::new();
        bt.fighters.insert(1, FighterState::default());
        bt.reset();
        assert!(bt.fighters.is_empty());
    }

    #[test]
    fn test_single_fight_backtest() {
        let mut bt = Backtester::new();
        let config = EloConfig::default();
        let fights = vec![create_test_fight(1, 2)];

        let result = bt.run(&fights, &config);

        assert_eq!(result.fights_processed, 1);
        assert_eq!(result.predictions.len(), 1);
        // Equal ratings -> 50% win probability
        assert!((result.predictions[0].predicted_probability - 0.5).abs() < 0.001);
    }

    #[test]
    fn test_multiple_fights() {
        let mut bt = Backtester::new();
        let config = EloConfig::default();

        // Fighter 1 beats fighter 2, then beats fighter 3
        let fights = vec![create_test_fight(1, 2), create_test_fight(1, 3)];

        let result = bt.run(&fights, &config);

        assert_eq!(result.fights_processed, 2);
        // Second prediction should show higher probability for fighter 1
        // (since they won the first fight and have higher rating now)
        assert!(result.predictions[1].predicted_probability > 0.5);
    }
}
