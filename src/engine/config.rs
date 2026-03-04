//! # Elo Configuration
//!
//! Defines configurable parameters for the Elo rating system.
//!
//! The `EloConfig` struct allows tuning of K-factors and multipliers
//! that affect how ratings change after each fight.

use std::fmt;

/// Configuration parameters for the Elo rating system.
///
/// These parameters control how much ratings change after fights
/// and how different fight contexts affect the magnitude of changes.
#[derive(Debug, Clone, PartialEq)]
pub struct EloConfig {
    /// Base K-factor for rating changes.
    /// Higher values mean more volatile ratings.
    pub k_factor: f64,

    /// Multiplier applied when a fight ends in a finish (KO/TKO/Submission).
    /// Values > 1.0 increase rating change for finishes.
    pub finish_multiplier: f64,

    /// Multiplier applied for title fights.
    /// Values > 1.0 increase rating change for title fights.
    pub title_fight_multiplier: f64,

    /// Multiplier applied for five-round fights (main events).
    /// Values > 1.0 increase rating change for five-round fights.
    pub five_round_multiplier: f64,
}

impl Default for EloConfig {
    /// Returns the default configuration with baseline values.
    fn default() -> Self {
        Self {
            k_factor: 32.0,
            finish_multiplier: 1.0,
            title_fight_multiplier: 1.0,
            five_round_multiplier: 1.0,
        }
    }
}

impl EloConfig {
    /// Creates a new EloConfig with the specified parameters.
    pub fn new(
        k_factor: f64,
        finish_multiplier: f64,
        title_fight_multiplier: f64,
        five_round_multiplier: f64,
    ) -> Self {
        Self {
            k_factor,
            finish_multiplier,
            title_fight_multiplier,
            five_round_multiplier,
        }
    }

    /// Creates an EloConfig with parameters optimized from historical data.
    ///
    /// These values can be updated after running optimization.
    #[allow(dead_code)]
    pub fn optimized() -> Self {
        Self {
            k_factor: 40.0,
            finish_multiplier: 1.1,
            title_fight_multiplier: 1.05,
            five_round_multiplier: 1.05,
        }
    }
}

impl fmt::Display for EloConfig {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "EloConfig {{ k_factor: {:.2}, finish_multiplier: {:.2}, \
             title_fight_multiplier: {:.2}, five_round_multiplier: {:.2} }}",
            self.k_factor,
            self.finish_multiplier,
            self.title_fight_multiplier,
            self.five_round_multiplier
        )
    }
}

/// Defines the parameter ranges for optimization.
///
/// This struct allows configuring which values to search over
/// during grid or random search optimization.
#[derive(Debug, Clone)]
pub struct ParameterRanges {
    /// K-factor values to test.
    pub k_factor: Vec<f64>,

    /// Finish multiplier values to test.
    pub finish_multiplier: Vec<f64>,

    /// Title fight multiplier values to test.
    pub title_fight_multiplier: Vec<f64>,

    /// Five-round fight multiplier values to test.
    pub five_round_multiplier: Vec<f64>,
}

impl Default for ParameterRanges {
    /// Returns default parameter ranges for optimization.
    fn default() -> Self {
        Self {
            k_factor: vec![20.0, 30.0, 40.0, 50.0, 60.0],
            finish_multiplier: vec![1.0, 1.1, 1.2, 1.3],
            title_fight_multiplier: vec![1.0, 1.05, 1.1, 1.15],
            five_round_multiplier: vec![1.0, 1.05, 1.1],
        }
    }
}

impl ParameterRanges {
    /// Creates new parameter ranges with custom values.
    #[allow(dead_code)]
    pub fn new(
        k_factor: Vec<f64>,
        finish_multiplier: Vec<f64>,
        title_fight_multiplier: Vec<f64>,
        five_round_multiplier: Vec<f64>,
    ) -> Self {
        Self {
            k_factor,
            finish_multiplier,
            title_fight_multiplier,
            five_round_multiplier,
        }
    }

    /// Returns the total number of combinations in the parameter space.
    pub fn total_combinations(&self) -> usize {
        self.k_factor.len()
            * self.finish_multiplier.len()
            * self.title_fight_multiplier.len()
            * self.five_round_multiplier.len()
    }

    /// Generates all possible EloConfig combinations from the parameter ranges.
    pub fn generate_all_configs(&self) -> Vec<EloConfig> {
        let mut configs = Vec::with_capacity(self.total_combinations());

        for &k in &self.k_factor {
            for &finish in &self.finish_multiplier {
                for &title in &self.title_fight_multiplier {
                    for &five_round in &self.five_round_multiplier {
                        configs.push(EloConfig::new(k, finish, title, five_round));
                    }
                }
            }
        }

        configs
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = EloConfig::default();
        assert_eq!(config.k_factor, 32.0);
        assert_eq!(config.finish_multiplier, 1.0);
        assert_eq!(config.title_fight_multiplier, 1.0);
        assert_eq!(config.five_round_multiplier, 1.0);
    }

    #[test]
    fn test_parameter_ranges_total() {
        let ranges = ParameterRanges::default();
        // 5 * 4 * 4 * 3 = 240
        assert_eq!(ranges.total_combinations(), 240);
    }

    #[test]
    fn test_generate_all_configs() {
        let ranges = ParameterRanges {
            k_factor: vec![20.0, 30.0],
            finish_multiplier: vec![1.0, 1.1],
            title_fight_multiplier: vec![1.0],
            five_round_multiplier: vec![1.0],
        };
        let configs = ranges.generate_all_configs();
        assert_eq!(configs.len(), 4);
    }
}
