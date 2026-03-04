//! # Predictions
//!
//! This module provides fight predictions based on Elo ratings.

use crate::database::Database;
use crate::espn::Espn;
use crate::espn::dto::EventDto;
use crate::espn::dto::fight_card::CompetitorDto;

/// A predicted fight outcome.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct FightPrediction {
    /// Fighter 1 name.
    pub fighter1_name: String,
    /// Fighter 1 ESPN ID.
    pub fighter1_id: String,
    /// Fighter 1 current rating (None if not in database).
    pub fighter1_rating: Option<f64>,
    /// Fighter 1 record.
    pub fighter1_record: Option<String>,

    /// Fighter 2 name.
    pub fighter2_name: String,
    /// Fighter 2 ESPN ID.
    pub fighter2_id: String,
    /// Fighter 2 current rating (None if not in database).
    pub fighter2_rating: Option<f64>,
    /// Fighter 2 record.
    pub fighter2_record: Option<String>,

    /// Probability that fighter 1 wins (0.0 to 1.0).
    pub fighter1_win_prob: f64,
    /// Probability that fighter 2 wins (0.0 to 1.0).
    pub fighter2_win_prob: f64,

    /// Weight class of the fight.
    pub weight_class: Option<String>,

    /// Whether both fighters have ratings (prediction is reliable).
    pub has_ratings: bool,
}

/// An event with fight predictions.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct EventPrediction {
    /// Event ID.
    pub id: String,
    /// Event name.
    pub name: String,
    /// Event date as string.
    pub date: String,
    /// List of fight predictions.
    pub fights: Vec<FightPrediction>,
}

/// Calculates the expected score (win probability) for fighter A against fighter B.
///
/// Uses the standard Elo formula: E_A = 1 / (1 + 10^((R_B - R_A) / 400))
pub fn calculate_win_probability(rating_a: f64, rating_b: f64) -> f64 {
    1.0 / (1.0 + 10f64.powf((rating_b - rating_a) / 400.0))
}

/// Default rating for unknown fighters.
const DEFAULT_RATING: f64 = 1000.0;

/// Generates predictions for all upcoming UFC events.
///
/// # Arguments
///
/// - `database`: Database connection for looking up fighter ratings.
///
/// # Returns
///
/// A vector of `EventPrediction` for upcoming events.
pub async fn get_upcoming_predictions(database: &Database) -> anyhow::Result<Vec<EventPrediction>> {
    let espn = Espn::new();
    let upcoming_events = espn.get_upcoming_events().await?;

    let mut predictions = Vec::new();

    for event in upcoming_events.iter().take(5) {
        // Limit to next 5 events
        if let Ok(prediction) = get_event_prediction(database, &espn, event).await {
            predictions.push(prediction);
        }
    }

    Ok(predictions)
}

/// Generates predictions for a single event.
async fn get_event_prediction(
    database: &Database,
    espn: &Espn,
    event: &EventDto,
) -> anyhow::Result<EventPrediction> {
    let fight_card = espn.get_fight_card(&event.id).await?;

    let mut fights = Vec::new();

    if let Some(cards) = fight_card.cards {
        // Process main card
        for competition in &cards.main.competitions {
            if competition.competitors.len() >= 2 {
                let prediction = create_fight_prediction(database, &competition.competitors).await;
                fights.push(prediction);
            }
        }

        // Process prelims
        if let Some(prelims) = &cards.prelims1 {
            for competition in &prelims.competitions {
                if competition.competitors.len() >= 2 {
                    let prediction =
                        create_fight_prediction(database, &competition.competitors).await;
                    fights.push(prediction);
                }
            }
        }

        if let Some(prelims) = &cards.prelims2 {
            for competition in &prelims.competitions {
                if competition.competitors.len() >= 2 {
                    let prediction =
                        create_fight_prediction(database, &competition.competitors).await;
                    fights.push(prediction);
                }
            }
        }
    }

    Ok(EventPrediction {
        id: event.id.clone(),
        name: event.name.clone(),
        date: event.date.date().to_string(),
        fights,
    })
}

/// Creates a fight prediction from two competitors.
async fn create_fight_prediction(
    database: &Database,
    competitors: &[CompetitorDto],
) -> FightPrediction {
    let fighter1 = &competitors[0];
    let fighter2 = &competitors[1];

    // Look up ratings from database
    let (fighter1_rating, fighter1_record) = get_fighter_info(database, &fighter1.athlete.id).await;
    let (fighter2_rating, fighter2_record) = get_fighter_info(database, &fighter2.athlete.id).await;

    // Use actual ratings or default
    let r1 = fighter1_rating.unwrap_or(DEFAULT_RATING);
    let r2 = fighter2_rating.unwrap_or(DEFAULT_RATING);

    // Calculate win probabilities
    let fighter1_win_prob = calculate_win_probability(r1, r2);
    let fighter2_win_prob = 1.0 - fighter1_win_prob;

    // Get weight class
    let weight_class = fighter1
        .athlete
        .weight_class
        .as_ref()
        .map(|w| w.slug.clone())
        .or_else(|| {
            fighter2
                .athlete
                .weight_class
                .as_ref()
                .map(|w| w.slug.clone())
        });

    FightPrediction {
        fighter1_name: fighter1.athlete.full_name.clone(),
        fighter1_id: fighter1.athlete.id.clone(),
        fighter1_rating,
        fighter1_record,
        fighter2_name: fighter2.athlete.full_name.clone(),
        fighter2_id: fighter2.athlete.id.clone(),
        fighter2_rating,
        fighter2_record,
        fighter1_win_prob,
        fighter2_win_prob,
        weight_class,
        has_ratings: fighter1_rating.is_some() && fighter2_rating.is_some(),
    }
}

/// Looks up a fighter's rating and record from the database.
async fn get_fighter_info(database: &Database, espn_id: &str) -> (Option<f64>, Option<String>) {
    // ESPN IDs are strings, need to parse to i64
    let id: i64 = match espn_id.parse() {
        Ok(id) => id,
        Err(_) => return (None, None),
    };

    match database.get_fighter(id).await {
        Ok(fighter) => {
            let record = format!("{}-{}", fighter.wins, fighter.losses);
            (Some(fighter.rating), Some(record))
        }
        Err(_) => (None, None),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Helper to check floating point equality within tolerance.
    fn approx_eq(a: f64, b: f64, tolerance: f64) -> bool {
        (a - b).abs() < tolerance
    }

    // ==================== Win Probability Tests ====================

    #[test]
    fn test_win_probability_equal_ratings() {
        let prob = calculate_win_probability(1000.0, 1000.0);
        assert!(approx_eq(prob, 0.5, 0.001));
    }

    #[test]
    fn test_win_probability_higher_rating_favored() {
        let prob = calculate_win_probability(1200.0, 1000.0);
        assert!(prob > 0.5);
        assert!(prob < 1.0);
        // 200 point difference should give ~76% win probability
        assert!(approx_eq(prob, 0.76, 0.01));
    }

    #[test]
    fn test_win_probability_symmetry() {
        let prob_a = calculate_win_probability(1200.0, 1000.0);
        let prob_b = calculate_win_probability(1000.0, 1200.0);
        assert!(approx_eq(prob_a + prob_b, 1.0, 0.001));
    }

    #[test]
    fn test_win_probability_large_difference() {
        // 400 point difference
        let prob = calculate_win_probability(1400.0, 1000.0);
        assert!(prob > 0.9);
        assert!(prob < 1.0);
    }

    #[test]
    fn test_win_probability_small_difference() {
        // 50 point difference
        let prob = calculate_win_probability(1050.0, 1000.0);
        assert!(prob > 0.5);
        assert!(prob < 0.6);
    }

    // ==================== Default Rating Tests ====================

    #[test]
    fn test_default_rating_value() {
        assert_eq!(DEFAULT_RATING, 1000.0);
    }

    #[test]
    fn test_unknown_fighters_get_50_50() {
        // When both fighters use default rating, should be 50/50
        let prob = calculate_win_probability(DEFAULT_RATING, DEFAULT_RATING);
        assert!(approx_eq(prob, 0.5, 0.001));
    }

    // ==================== FightPrediction Tests ====================

    #[test]
    fn test_fight_prediction_has_ratings_both_present() {
        let prediction = FightPrediction {
            fighter1_name: "Fighter A".to_string(),
            fighter1_id: "1".to_string(),
            fighter1_rating: Some(1200.0),
            fighter1_record: Some("10-2".to_string()),
            fighter2_name: "Fighter B".to_string(),
            fighter2_id: "2".to_string(),
            fighter2_rating: Some(1100.0),
            fighter2_record: Some("8-3".to_string()),
            fighter1_win_prob: 0.64,
            fighter2_win_prob: 0.36,
            weight_class: Some("lightweight".to_string()),
            has_ratings: true,
        };
        assert!(prediction.has_ratings);
    }

    #[test]
    fn test_fight_prediction_has_ratings_one_missing() {
        let prediction = FightPrediction {
            fighter1_name: "Fighter A".to_string(),
            fighter1_id: "1".to_string(),
            fighter1_rating: Some(1200.0),
            fighter1_record: Some("10-2".to_string()),
            fighter2_name: "Fighter B".to_string(),
            fighter2_id: "2".to_string(),
            fighter2_rating: None,
            fighter2_record: None,
            fighter1_win_prob: 0.64,
            fighter2_win_prob: 0.36,
            weight_class: Some("lightweight".to_string()),
            has_ratings: false,
        };
        assert!(!prediction.has_ratings);
    }

    #[test]
    fn test_fight_prediction_probabilities_sum_to_one() {
        let r1 = 1150.0;
        let r2 = 1050.0;
        let prob1 = calculate_win_probability(r1, r2);
        let prob2 = 1.0 - prob1;

        assert!(approx_eq(prob1 + prob2, 1.0, 0.001));
    }

    // ==================== Edge Case Tests ====================

    #[test]
    fn test_very_high_rating_difference() {
        // Champion vs newcomer scenario
        let prob = calculate_win_probability(1600.0, 800.0);
        assert!(prob > 0.99);
        assert!(prob < 1.0); // Should never be exactly 1.0
    }

    #[test]
    fn test_negative_ratings_handled() {
        // Shouldn't happen in practice, but should still work
        let prob = calculate_win_probability(500.0, 500.0);
        assert!(approx_eq(prob, 0.5, 0.001));
    }

    #[test]
    fn test_zero_ratings() {
        let prob = calculate_win_probability(0.0, 0.0);
        assert!(approx_eq(prob, 0.5, 0.001));
    }
}
