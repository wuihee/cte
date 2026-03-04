//! # Elo Configuration
//!
//! Defines configurable parameters for the Elo rating system.
//!
//! The `EloConfig` struct allows tuning of K-factors and multipliers
//! that affect how ratings change after each fight.

use std::fmt;
use std::fs;
use std::path::Path;

use serde::{Deserialize, Serialize};

/// Default path for the config file.
pub const CONFIG_FILE_PATH: &str = "data/elo_config.json";

/// Configuration parameters for the Elo rating system.
///
/// These parameters control how much ratings change after fights
/// and how different fight contexts affect the magnitude of changes.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
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

    /// Loads the configuration from the default file path.
    ///
    /// If the file doesn't exist, returns the default configuration.
    pub fn load() -> Self {
        Self::load_from(CONFIG_FILE_PATH)
    }

    /// Loads the configuration from a specific file path.
    ///
    /// If the file doesn't exist or is invalid, returns the default configuration.
    pub fn load_from<P: AsRef<Path>>(path: P) -> Self {
        match fs::read_to_string(path) {
            Ok(contents) => serde_json::from_str(&contents).unwrap_or_default(),
            Err(_) => Self::default(),
        }
    }

    /// Saves the configuration to the default file path.
    ///
    /// Creates the parent directory if it doesn't exist.
    pub fn save(&self) -> std::io::Result<()> {
        self.save_to(CONFIG_FILE_PATH)
    }

    /// Saves the configuration to a specific file path.
    ///
    /// Creates the parent directory if it doesn't exist.
    pub fn save_to<P: AsRef<Path>>(&self, path: P) -> std::io::Result<()> {
        let path = path.as_ref();

        // Create parent directory if needed
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }

        let json = serde_json::to_string_pretty(self)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;

        fs::write(path, json)
    }

    /// Checks if a saved configuration exists.
    pub fn exists() -> bool {
        Path::new(CONFIG_FILE_PATH).exists()
    }

    /// Deletes the saved configuration file (resets to default).
    pub fn reset() -> std::io::Result<()> {
        if Self::exists() {
            fs::remove_file(CONFIG_FILE_PATH)
        } else {
            Ok(())
        }
    }

    /// Returns a short summary string for display in UI.
    pub fn summary(&self) -> String {
        format!(
            "K={:.0} F={:.2} T={:.2} 5R={:.2}",
            self.k_factor,
            self.finish_multiplier,
            self.title_fight_multiplier,
            self.five_round_multiplier
        )
    }

    /// Checks if this config differs from the default.
    pub fn is_custom(&self) -> bool {
        *self != Self::default()
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
    use std::fs;
    use std::path::PathBuf;

    fn test_config_path() -> PathBuf {
        PathBuf::from("test_elo_config.json")
    }

    #[test]
    fn test_default_config() {
        let config = EloConfig::default();
        assert_eq!(config.k_factor, 32.0);
        assert_eq!(config.finish_multiplier, 1.0);
        assert_eq!(config.title_fight_multiplier, 1.0);
        assert_eq!(config.five_round_multiplier, 1.0);
    }

    #[test]
    fn test_config_serialization() {
        let config = EloConfig::new(50.0, 1.2, 1.1, 1.05);
        let json = serde_json::to_string(&config).unwrap();
        let loaded: EloConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(config, loaded);
    }

    #[test]
    fn test_config_save_load() {
        let path = test_config_path();
        let config = EloConfig::new(60.0, 1.3, 1.15, 1.1);

        // Save
        config.save_to(&path).unwrap();

        // Load
        let loaded = EloConfig::load_from(&path);
        assert_eq!(config, loaded);

        // Cleanup
        fs::remove_file(&path).ok();
    }

    #[test]
    fn test_load_missing_file() {
        let config = EloConfig::load_from("nonexistent_file.json");
        assert_eq!(config, EloConfig::default());
    }

    #[test]
    fn test_is_custom() {
        let default = EloConfig::default();
        assert!(!default.is_custom());

        let custom = EloConfig::new(50.0, 1.2, 1.1, 1.05);
        assert!(custom.is_custom());
    }

    #[test]
    fn test_summary() {
        let config = EloConfig::new(50.0, 1.20, 1.10, 1.05);
        assert_eq!(config.summary(), "K=50 F=1.20 T=1.10 5R=1.05");
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
