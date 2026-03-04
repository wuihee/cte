//! # Parameter Optimizer
//!
//! Automatically searches for the best Elo configuration by evaluating
//! different parameter combinations on historical fight data.
//!
//! Supports:
//! - Grid search: exhaustive search over all combinations
//! - Random search: random sampling from parameter space

use std::fs::File;
use std::io::Write;
use std::path::Path;

use crate::database::entities::Fight;
use crate::engine::backtest::Backtester;
use crate::engine::config::{EloConfig, ParameterRanges};
use crate::engine::metrics::EvaluationMetrics;

/// Result of evaluating a single configuration.
#[derive(Debug, Clone)]
pub struct OptimizationResult {
    /// The configuration that was tested.
    pub config: EloConfig,

    /// Log loss achieved by this configuration.
    pub log_loss: f64,

    /// Brier score achieved by this configuration.
    pub brier_score: f64,

    /// Accuracy achieved by this configuration.
    pub accuracy: f64,

    /// Number of predictions evaluated.
    pub num_predictions: usize,
}

impl OptimizationResult {
    /// Creates a new optimization result from config and metrics.
    pub fn new(config: EloConfig, metrics: EvaluationMetrics) -> Self {
        Self {
            config,
            log_loss: metrics.log_loss,
            brier_score: metrics.brier_score,
            accuracy: metrics.accuracy,
            num_predictions: metrics.num_predictions,
        }
    }
}

/// The best configuration found by the optimizer.
#[derive(Debug, Clone)]
pub struct BestConfiguration {
    /// The optimal configuration.
    pub config: EloConfig,

    /// Log loss of the best configuration.
    pub log_loss: f64,

    /// Brier score of the best configuration.
    pub brier_score: f64,

    /// Accuracy of the best configuration.
    pub accuracy: f64,
}

impl From<&OptimizationResult> for BestConfiguration {
    fn from(result: &OptimizationResult) -> Self {
        Self {
            config: result.config.clone(),
            log_loss: result.log_loss,
            brier_score: result.brier_score,
            accuracy: result.accuracy,
        }
    }
}

impl std::fmt::Display for BestConfiguration {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Best Configuration:")?;
        writeln!(f, "  K-Factor:              {:.2}", self.config.k_factor)?;
        writeln!(
            f,
            "  Finish Multiplier:     {:.2}",
            self.config.finish_multiplier
        )?;
        writeln!(
            f,
            "  Title Fight Multiplier:{:.2}",
            self.config.title_fight_multiplier
        )?;
        writeln!(
            f,
            "  Five Round Multiplier: {:.2}",
            self.config.five_round_multiplier
        )?;
        writeln!(f)?;
        writeln!(f, "Performance Metrics:")?;
        writeln!(f, "  Log Loss:    {:.4}", self.log_loss)?;
        writeln!(f, "  Brier Score: {:.4}", self.brier_score)?;
        writeln!(f, "  Accuracy:    {:.2}%", self.accuracy * 100.0)?;
        Ok(())
    }
}

/// Callback type for progress updates.
pub type ProgressCallback = Box<dyn Fn(usize, usize, &EloConfig) + Send>;

/// Parameter optimizer using grid or random search.
pub struct Optimizer {
    /// Parameter ranges to search over.
    ranges: ParameterRanges,

    /// Progress callback (optional).
    progress_callback: Option<ProgressCallback>,
}

impl Optimizer {
    /// Creates a new optimizer with default parameter ranges.
    pub fn new() -> Self {
        Self {
            ranges: ParameterRanges::default(),
            progress_callback: None,
        }
    }

    /// Creates a new optimizer with custom parameter ranges.
    pub fn with_ranges(ranges: ParameterRanges) -> Self {
        Self {
            ranges,
            progress_callback: None,
        }
    }

    /// Sets a progress callback for tracking optimization progress.
    pub fn with_progress<F>(mut self, callback: F) -> Self
    where
        F: Fn(usize, usize, &EloConfig) + Send + 'static,
    {
        self.progress_callback = Some(Box::new(callback));
        self
    }

    /// Returns the total number of configurations to test.
    pub fn total_configurations(&self) -> usize {
        self.ranges.total_combinations()
    }

    /// Runs grid search optimization.
    ///
    /// Evaluates every combination of parameters and returns results
    /// sorted by log loss (ascending).
    ///
    /// # Arguments
    ///
    /// * `fights` - Historical fights to evaluate on (sorted by date).
    ///
    /// # Returns
    ///
    /// A tuple of (best_configuration, all_results_sorted_by_log_loss).
    pub fn grid_search(&self, fights: &[Fight]) -> (BestConfiguration, Vec<OptimizationResult>) {
        let configs = self.ranges.generate_all_configs();
        let total = configs.len();

        let mut results = Vec::with_capacity(total);
        let mut backtester = Backtester::new();

        for (i, config) in configs.iter().enumerate() {
            // Report progress
            if let Some(ref callback) = self.progress_callback {
                callback(i + 1, total, config);
            }

            // Run backtest
            let backtest_result = backtester.run(fights, config);
            results.push(OptimizationResult::new(
                config.clone(),
                backtest_result.metrics,
            ));
        }

        // Sort by log loss (ascending)
        results.sort_by(|a, b| {
            a.log_loss
                .partial_cmp(&b.log_loss)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        // Get best configuration
        let best = BestConfiguration::from(&results[0]);

        (best, results)
    }

    /// Runs random search optimization.
    ///
    /// Samples random configurations from the parameter space and
    /// returns the best one found.
    ///
    /// # Arguments
    ///
    /// * `fights` - Historical fights to evaluate on (sorted by date).
    /// * `num_samples` - Number of random configurations to test.
    /// * `seed` - Random seed for reproducibility.
    ///
    /// # Returns
    ///
    /// A tuple of (best_configuration, all_results_sorted_by_log_loss).
    pub fn random_search(
        &self,
        fights: &[Fight],
        num_samples: usize,
        seed: u64,
    ) -> (BestConfiguration, Vec<OptimizationResult>) {
        let configs = self.generate_random_configs(num_samples, seed);
        let total = configs.len();

        let mut results = Vec::with_capacity(total);
        let mut backtester = Backtester::new();

        for (i, config) in configs.iter().enumerate() {
            // Report progress
            if let Some(ref callback) = self.progress_callback {
                callback(i + 1, total, config);
            }

            // Run backtest
            let backtest_result = backtester.run(fights, config);
            results.push(OptimizationResult::new(
                config.clone(),
                backtest_result.metrics,
            ));
        }

        // Sort by log loss (ascending)
        results.sort_by(|a, b| {
            a.log_loss
                .partial_cmp(&b.log_loss)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        // Get best configuration
        let best = BestConfiguration::from(&results[0]);

        (best, results)
    }

    /// Generates random configurations from the parameter space.
    fn generate_random_configs(&self, num_samples: usize, seed: u64) -> Vec<EloConfig> {
        // Simple LCG random number generator for reproducibility
        let mut rng = LcgRng::new(seed);
        let mut configs = Vec::with_capacity(num_samples);

        for _ in 0..num_samples {
            let k = self.ranges.k_factor[rng.next_usize() % self.ranges.k_factor.len()];
            let finish = self.ranges.finish_multiplier
                [rng.next_usize() % self.ranges.finish_multiplier.len()];
            let title = self.ranges.title_fight_multiplier
                [rng.next_usize() % self.ranges.title_fight_multiplier.len()];
            let five_round = self.ranges.five_round_multiplier
                [rng.next_usize() % self.ranges.five_round_multiplier.len()];

            configs.push(EloConfig::new(k, finish, title, five_round));
        }

        configs
    }
}

impl Default for Optimizer {
    fn default() -> Self {
        Self::new()
    }
}

/// Simple Linear Congruential Generator for reproducible random numbers.
struct LcgRng {
    state: u64,
}

impl LcgRng {
    fn new(seed: u64) -> Self {
        Self { state: seed }
    }

    fn next_u64(&mut self) -> u64 {
        // LCG parameters from Numerical Recipes
        self.state = self.state.wrapping_mul(6364136223846793005).wrapping_add(1);
        self.state
    }

    fn next_usize(&mut self) -> usize {
        self.next_u64() as usize
    }
}

/// Exports optimization results to a CSV file.
///
/// # Arguments
///
/// * `results` - Optimization results to export.
/// * `path` - Path to the output CSV file.
///
/// # Returns
///
/// `Ok(())` on success, or an error if writing fails.
pub fn export_results_to_csv<P: AsRef<Path>>(
    results: &[OptimizationResult],
    path: P,
) -> std::io::Result<()> {
    let mut file = File::create(path)?;

    // Write header
    writeln!(
        file,
        "k_factor,finish_multiplier,title_fight_multiplier,five_round_multiplier,log_loss,brier_score,accuracy,num_predictions"
    )?;

    // Write data rows
    for result in results {
        writeln!(
            file,
            "{:.2},{:.2},{:.2},{:.2},{:.6},{:.6},{:.6},{}",
            result.config.k_factor,
            result.config.finish_multiplier,
            result.config.title_fight_multiplier,
            result.config.five_round_multiplier,
            result.log_loss,
            result.brier_score,
            result.accuracy,
            result.num_predictions,
        )?;
    }

    Ok(())
}

/// Prints a summary of the top N configurations.
pub fn print_top_results(results: &[OptimizationResult], n: usize) {
    println!("\nTop {} Configurations by Log Loss:", n.min(results.len()));
    println!("{:-<100}", "");
    println!(
        "{:>6} {:>8} {:>8} {:>8} {:>10} {:>10} {:>10}",
        "K", "Finish", "Title", "5Round", "Log Loss", "Brier", "Accuracy"
    );
    println!("{:-<100}", "");

    for result in results.iter().take(n) {
        println!(
            "{:>6.1} {:>8.2} {:>8.2} {:>8.2} {:>10.4} {:>10.4} {:>9.2}%",
            result.config.k_factor,
            result.config.finish_multiplier,
            result.config.title_fight_multiplier,
            result.config.five_round_multiplier,
            result.log_loss,
            result.brier_score,
            result.accuracy * 100.0,
        );
    }
    println!("{:-<100}", "");
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::database::entities::Fight;
    use time::OffsetDateTime;

    fn create_test_fights() -> Vec<Fight> {
        (0..10)
            .map(|i| Fight {
                id: i,
                event_id: 1,
                winner_id: i * 2,
                loser_id: i * 2 + 1,
                date: OffsetDateTime::now_utc(),
                fight_time: 300,
                weight_class: Some("lightweight".to_string()),
                finish_method: Some("Decision".to_string()),
            })
            .collect()
    }

    #[test]
    fn test_optimizer_total_configurations() {
        let optimizer = Optimizer::new();
        // Default: 5 * 4 * 4 * 3 = 240
        assert_eq!(optimizer.total_configurations(), 240);
    }

    #[test]
    fn test_grid_search() {
        let ranges = ParameterRanges {
            k_factor: vec![30.0, 40.0],
            finish_multiplier: vec![1.0, 1.1],
            title_fight_multiplier: vec![1.0],
            five_round_multiplier: vec![1.0],
        };
        let optimizer = Optimizer::with_ranges(ranges);
        let fights = create_test_fights();

        let (best, results) = optimizer.grid_search(&fights);

        assert_eq!(results.len(), 4);
        // Results should be sorted by log loss
        for i in 1..results.len() {
            assert!(results[i - 1].log_loss <= results[i].log_loss);
        }
        // Best should match first result
        assert_eq!(best.log_loss, results[0].log_loss);
    }

    #[test]
    fn test_random_search() {
        let optimizer = Optimizer::new();
        let fights = create_test_fights();

        let (best, results) = optimizer.random_search(&fights, 10, 42);

        assert_eq!(results.len(), 10);
        assert_eq!(best.log_loss, results[0].log_loss);
    }

    #[test]
    fn test_random_search_reproducibility() {
        let optimizer = Optimizer::new();
        let fights = create_test_fights();

        let (best1, _) = optimizer.random_search(&fights, 5, 42);
        let (best2, _) = optimizer.random_search(&fights, 5, 42);

        // Same seed should produce same results
        assert_eq!(best1.log_loss, best2.log_loss);
        assert_eq!(best1.config.k_factor, best2.config.k_factor);
    }
}
