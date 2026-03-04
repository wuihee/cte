//! # Evaluation Metrics
//!
//! Provides metrics for evaluating prediction quality.
//!
//! This module implements standard evaluation metrics for probabilistic predictions:
//! - Log Loss (cross-entropy loss)
//! - Brier Score
//! - Accuracy

use std::fmt;

/// A single prediction record for evaluation.
#[derive(Debug, Clone)]
pub struct PredictionRecord {
    /// Predicted probability that fighter A (the winner) wins.
    /// Should be between 0.0 and 1.0.
    pub predicted_probability: f64,

    /// Actual outcome: 1.0 if fighter A won, 0.0 if fighter B won.
    /// For this system, we always predict from the winner's perspective,
    /// so this is always 1.0 during backtesting.
    pub actual_outcome: f64,
}

impl PredictionRecord {
    /// Creates a new prediction record.
    #[allow(dead_code)]
    pub fn new(predicted_probability: f64, actual_outcome: f64) -> Self {
        Self {
            predicted_probability,
            actual_outcome,
        }
    }

    /// Creates a prediction record for a correct prediction (winner had probability p).
    pub fn winner_prediction(win_probability: f64) -> Self {
        Self {
            predicted_probability: win_probability,
            actual_outcome: 1.0,
        }
    }
}

/// Aggregated evaluation metrics for a set of predictions.
#[derive(Debug, Clone)]
pub struct EvaluationMetrics {
    /// Log loss (cross-entropy loss). Lower is better.
    pub log_loss: f64,

    /// Brier score. Lower is better. Range: [0, 1].
    pub brier_score: f64,

    /// Accuracy (proportion of correct predictions). Higher is better.
    pub accuracy: f64,

    /// Number of predictions evaluated.
    pub num_predictions: usize,
}

impl Default for EvaluationMetrics {
    fn default() -> Self {
        Self {
            log_loss: f64::INFINITY,
            brier_score: 1.0,
            accuracy: 0.0,
            num_predictions: 0,
        }
    }
}

impl fmt::Display for EvaluationMetrics {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Metrics {{ log_loss: {:.4}, brier_score: {:.4}, accuracy: {:.2}%, n: {} }}",
            self.log_loss,
            self.brier_score,
            self.accuracy * 100.0,
            self.num_predictions
        )
    }
}

/// Calculates evaluation metrics from a collection of predictions.
///
/// # Arguments
///
/// * `predictions` - Slice of prediction records to evaluate.
///
/// # Returns
///
/// `EvaluationMetrics` containing log loss, Brier score, and accuracy.
pub fn calculate_metrics(predictions: &[PredictionRecord]) -> EvaluationMetrics {
    if predictions.is_empty() {
        return EvaluationMetrics::default();
    }

    let n = predictions.len() as f64;
    let mut log_loss_sum = 0.0;
    let mut brier_sum = 0.0;
    let mut correct_count = 0;

    for pred in predictions {
        // Clamp probability to avoid log(0)
        let p = pred.predicted_probability.clamp(1e-15, 1.0 - 1e-15);
        let y = pred.actual_outcome;

        // Log loss: -[y * log(p) + (1-y) * log(1-p)]
        log_loss_sum += -(y * p.ln() + (1.0 - y) * (1.0 - p).ln());

        // Brier score: (p - y)^2
        brier_sum += (p - y).powi(2);

        // Accuracy: prediction is correct if p > 0.5 and y = 1, or p < 0.5 and y = 0
        let predicted_win = p > 0.5;
        let actual_win = y > 0.5;
        if predicted_win == actual_win {
            correct_count += 1;
        }
    }

    EvaluationMetrics {
        log_loss: log_loss_sum / n,
        brier_score: brier_sum / n,
        accuracy: correct_count as f64 / n,
        num_predictions: predictions.len(),
    }
}

/// Calculates log loss for a single prediction.
///
/// # Arguments
///
/// * `predicted` - Predicted probability (0 to 1).
/// * `actual` - Actual outcome (0 or 1).
///
/// # Returns
///
/// The log loss for this prediction.
#[allow(dead_code)]
pub fn log_loss(predicted: f64, actual: f64) -> f64 {
    let p = predicted.clamp(1e-15, 1.0 - 1e-15);
    -(actual * p.ln() + (1.0 - actual) * (1.0 - p).ln())
}

/// Calculates Brier score for a single prediction.
///
/// # Arguments
///
/// * `predicted` - Predicted probability (0 to 1).
/// * `actual` - Actual outcome (0 or 1).
///
/// # Returns
///
/// The Brier score for this prediction.
#[allow(dead_code)]
pub fn brier_score(predicted: f64, actual: f64) -> f64 {
    (predicted - actual).powi(2)
}

/// Checks if a prediction was correct.
///
/// # Arguments
///
/// * `predicted` - Predicted probability (0 to 1).
/// * `actual` - Actual outcome (0 or 1).
///
/// # Returns
///
/// `true` if the prediction was correct.
#[allow(dead_code)]
pub fn is_correct(predicted: f64, actual: f64) -> bool {
    (predicted > 0.5) == (actual > 0.5)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_log_loss_perfect_prediction() {
        // Perfect prediction: predicted 1.0, actual 1.0
        let ll = log_loss(0.999, 1.0);
        assert!(ll < 0.01);
    }

    #[test]
    fn test_log_loss_bad_prediction() {
        // Bad prediction: predicted 0.1, actual 1.0
        let ll = log_loss(0.1, 1.0);
        assert!(ll > 2.0);
    }

    #[test]
    fn test_brier_score_perfect() {
        let bs = brier_score(1.0, 1.0);
        assert!((bs - 0.0).abs() < 0.001);
    }

    #[test]
    fn test_brier_score_worst() {
        let bs = brier_score(0.0, 1.0);
        assert!((bs - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_accuracy() {
        assert!(is_correct(0.6, 1.0));
        assert!(is_correct(0.4, 0.0));
        assert!(!is_correct(0.4, 1.0));
        assert!(!is_correct(0.6, 0.0));
    }

    #[test]
    fn test_calculate_metrics() {
        let predictions = vec![
            PredictionRecord::new(0.7, 1.0), // Correct
            PredictionRecord::new(0.6, 1.0), // Correct
            PredictionRecord::new(0.4, 1.0), // Incorrect
            PredictionRecord::new(0.3, 0.0), // Correct
        ];

        let metrics = calculate_metrics(&predictions);
        assert_eq!(metrics.num_predictions, 4);
        assert!((metrics.accuracy - 0.75).abs() < 0.001);
    }

    #[test]
    fn test_empty_predictions() {
        let metrics = calculate_metrics(&[]);
        assert_eq!(metrics.num_predictions, 0);
        assert!(metrics.log_loss.is_infinite());
    }
}
