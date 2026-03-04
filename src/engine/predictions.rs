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
