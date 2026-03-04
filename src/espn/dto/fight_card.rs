//! # Fight Card DTO
//!
//! DTO for the fight card endpoint.

use serde::Deserialize;

/// Root UFC event payload.
#[derive(Debug, Deserialize)]
pub struct FightCardDto {
    pub cards: Option<CardsDto>,
}

/// Collection of card segments (main, prelims, early prelims).
#[derive(Debug, Deserialize)]
pub struct CardsDto {
    pub main: CardDto,
    pub prelims1: Option<CardDto>,
    pub prelims2: Option<CardDto>,
}

/// A single card segment such as Main Card.
#[derive(Debug, Deserialize)]
pub struct CardDto {
    pub competitions: Vec<CompetitionDto>,
}

/// A single fight entry.
#[derive(Debug, Deserialize)]
pub struct CompetitionDto {
    pub id: String,
    pub status: StatusDto,
    pub competitors: Vec<CompetitorDto>,
}

#[derive(Debug, Deserialize)]
#[serde(default)]
pub struct StatusDto {
    pub clock: String,
    pub period: u32,
    pub result: ResultDto,
}

impl Default for StatusDto {
    fn default() -> Self {
        Self {
            clock: "0".to_string(),
            period: 0,
            result: ResultDto::default(),
        }
    }
}

#[derive(Debug, Deserialize, Default)]
pub struct ResultDto {
    /// kotko, sub, u-dec, s-dec, no contest, draw, etc.
    #[serde(default)]
    pub name: String,
}

/// Information about a competitor of a fight.
#[derive(Debug, Deserialize)]
pub struct CompetitorDto {
    pub athlete: AthleteDto,
    #[serde(default)]
    pub winner: bool,
    #[serde(default)]
    pub stats: Vec<StatsDto>,
}

/// Fighter information.
#[derive(Debug, Deserialize, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct AthleteDto {
    pub id: String,
    pub full_name: String,
    pub weight_class: Option<WeightClassDto>,
}

#[derive(Debug, Deserialize)]
pub struct WeightClassDto {
    /// Lightweight, etc.
    #[serde(default)]
    pub slug: String,
}

#[derive(Debug, Deserialize, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct StatsDto {
    /// Significant strikes, time in control, etc.
    pub name: String,
    pub value: f64,
    pub display_value: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    // ==================== Deserialization Tests ====================

    #[test]
    fn test_deserialize_stats_dto_complete() {
        let json = r#"{
            "name": "sigStrikes",
            "value": 45.0,
            "displayValue": "45/80"
        }"#;

        let stats: StatsDto = serde_json::from_str(json).unwrap();
        assert_eq!(stats.name, "sigStrikes");
        assert_eq!(stats.value, 45.0);
        assert_eq!(stats.display_value, "45/80");
    }

    #[test]
    fn test_deserialize_stats_dto_missing_value() {
        let json = r#"{
            "name": "sigStrikes",
            "displayValue": "45/80"
        }"#;

        let stats: StatsDto = serde_json::from_str(json).unwrap();
        assert_eq!(stats.name, "sigStrikes");
        assert_eq!(stats.value, 0.0); // Default
        assert_eq!(stats.display_value, "45/80");
    }

    #[test]
    fn test_deserialize_stats_dto_empty() {
        let json = r#"{}"#;

        let stats: StatsDto = serde_json::from_str(json).unwrap();
        assert_eq!(stats.name, "");
        assert_eq!(stats.value, 0.0);
        assert_eq!(stats.display_value, "");
    }

    #[test]
    fn test_deserialize_athlete_dto_complete() {
        let json = r#"{
            "id": "12345",
            "fullName": "Jon Jones",
            "weightClass": {
                "slug": "light-heavyweight"
            }
        }"#;

        let athlete: AthleteDto = serde_json::from_str(json).unwrap();
        assert_eq!(athlete.id, "12345");
        assert_eq!(athlete.full_name, "Jon Jones");
        assert!(athlete.weight_class.is_some());
        assert_eq!(athlete.weight_class.unwrap().slug, "light-heavyweight");
    }

    #[test]
    fn test_deserialize_athlete_dto_no_weight_class() {
        let json = r#"{
            "id": "12345",
            "fullName": "Jon Jones"
        }"#;

        let athlete: AthleteDto = serde_json::from_str(json).unwrap();
        assert_eq!(athlete.id, "12345");
        assert_eq!(athlete.full_name, "Jon Jones");
        assert!(athlete.weight_class.is_none());
    }

    #[test]
    fn test_deserialize_athlete_dto_empty() {
        let json = r#"{}"#;

        let athlete: AthleteDto = serde_json::from_str(json).unwrap();
        assert_eq!(athlete.id, "");
        assert_eq!(athlete.full_name, "");
        assert!(athlete.weight_class.is_none());
    }

    #[test]
    fn test_deserialize_result_dto_complete() {
        let json = r#"{"name": "KO/TKO"}"#;

        let result: ResultDto = serde_json::from_str(json).unwrap();
        assert_eq!(result.name, "KO/TKO");
    }

    #[test]
    fn test_deserialize_result_dto_empty() {
        let json = r#"{}"#;

        let result: ResultDto = serde_json::from_str(json).unwrap();
        assert_eq!(result.name, "");
    }

    #[test]
    fn test_deserialize_status_dto_complete() {
        let json = r#"{
            "clock": "4:32",
            "period": 2,
            "result": {"name": "U-DEC"}
        }"#;

        let status: StatusDto = serde_json::from_str(json).unwrap();
        assert_eq!(status.clock, "4:32");
        assert_eq!(status.period, 2);
        assert_eq!(status.result.name, "U-DEC");
    }

    #[test]
    fn test_deserialize_status_dto_defaults() {
        let json = r#"{}"#;

        let status: StatusDto = serde_json::from_str(json).unwrap();
        assert_eq!(status.clock, "0");
        assert_eq!(status.period, 0);
        assert_eq!(status.result.name, "");
    }

    #[test]
    fn test_deserialize_competitor_dto() {
        let json = r#"{
            "athlete": {
                "id": "123",
                "fullName": "Fighter A"
            },
            "winner": true,
            "stats": [
                {"name": "knockDowns", "value": 2.0, "displayValue": "2"}
            ]
        }"#;

        let competitor: CompetitorDto = serde_json::from_str(json).unwrap();
        assert_eq!(competitor.athlete.id, "123");
        assert_eq!(competitor.athlete.full_name, "Fighter A");
        assert!(competitor.winner);
        assert_eq!(competitor.stats.len(), 1);
        assert_eq!(competitor.stats[0].name, "knockDowns");
    }

    #[test]
    fn test_deserialize_competitor_dto_no_stats() {
        let json = r#"{
            "athlete": {
                "id": "123",
                "fullName": "Fighter A"
            },
            "winner": false
        }"#;

        let competitor: CompetitorDto = serde_json::from_str(json).unwrap();
        assert!(!competitor.winner);
        assert!(competitor.stats.is_empty());
    }

    #[test]
    fn test_deserialize_card_dto() {
        let json = r#"{
            "competitions": [
                {
                    "id": "1",
                    "status": {"clock": "0", "period": 3, "result": {"name": "U-DEC"}},
                    "competitors": [
                        {"athlete": {"id": "1", "fullName": "A"}, "winner": true, "stats": []},
                        {"athlete": {"id": "2", "fullName": "B"}, "winner": false, "stats": []}
                    ]
                }
            ]
        }"#;

        let card: CardDto = serde_json::from_str(json).unwrap();
        assert_eq!(card.competitions.len(), 1);
        assert_eq!(card.competitions[0].id, "1");
        assert_eq!(card.competitions[0].competitors.len(), 2);
    }

    #[test]
    fn test_deserialize_fight_card_dto_with_cards() {
        let json = r#"{
            "cards": {
                "main": {
                    "competitions": []
                },
                "prelims1": null,
                "prelims2": null
            }
        }"#;

        let fight_card: FightCardDto = serde_json::from_str(json).unwrap();
        assert!(fight_card.cards.is_some());
        let cards = fight_card.cards.unwrap();
        assert!(cards.prelims1.is_none());
        assert!(cards.prelims2.is_none());
    }

    #[test]
    fn test_deserialize_fight_card_dto_no_cards() {
        let json = r#"{}"#;

        let fight_card: FightCardDto = serde_json::from_str(json).unwrap();
        assert!(fight_card.cards.is_none());
    }

    // ==================== Weight Class Tests ====================

    #[test]
    fn test_deserialize_weight_class_dto() {
        let json = r#"{"slug": "heavyweight"}"#;

        let wc: WeightClassDto = serde_json::from_str(json).unwrap();
        assert_eq!(wc.slug, "heavyweight");
    }

    #[test]
    fn test_deserialize_weight_class_dto_empty_slug() {
        let json = r#"{"slug": ""}"#;

        let wc: WeightClassDto = serde_json::from_str(json).unwrap();
        assert_eq!(wc.slug, "");
    }
}
