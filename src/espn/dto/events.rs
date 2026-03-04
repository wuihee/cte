//! # Events DTO
//!
//! DTO for the endpoint to get all events.

use serde::Deserialize;
use time::OffsetDateTime;

/// Represents a list of UFC events.
#[derive(Debug, Deserialize)]
pub struct EventsDto {
    pub items: Vec<EventDto>,
}

/// Represents a single UFC event.
#[derive(Debug, Clone, Deserialize)]
pub struct EventDto {
    pub id: String,

    #[serde(with = "time::serde::iso8601")]
    pub date: OffsetDateTime,

    pub name: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deserialize_event_dto() {
        let json = r#"{
            "id": "401234567",
            "date": "2024-12-14T22:00:00Z",
            "name": "UFC 310: Pantoja vs. Asakura"
        }"#;

        let event: EventDto = serde_json::from_str(json).unwrap();
        assert_eq!(event.id, "401234567");
        assert_eq!(event.name, "UFC 310: Pantoja vs. Asakura");
        assert_eq!(event.date.year(), 2024);
        assert_eq!(event.date.month() as u8, 12);
        assert_eq!(event.date.day(), 14);
    }

    #[test]
    fn test_deserialize_events_dto() {
        let json = r#"{
            "items": [
                {
                    "id": "1",
                    "date": "2024-01-01T00:00:00Z",
                    "name": "Event 1"
                },
                {
                    "id": "2",
                    "date": "2024-02-01T00:00:00Z",
                    "name": "Event 2"
                }
            ]
        }"#;

        let events: EventsDto = serde_json::from_str(json).unwrap();
        assert_eq!(events.items.len(), 2);
        assert_eq!(events.items[0].id, "1");
        assert_eq!(events.items[1].id, "2");
    }

    #[test]
    fn test_deserialize_events_dto_empty() {
        let json = r#"{"items": []}"#;

        let events: EventsDto = serde_json::from_str(json).unwrap();
        assert!(events.items.is_empty());
    }

    #[test]
    fn test_event_dto_clone() {
        let json = r#"{
            "id": "123",
            "date": "2024-06-15T20:00:00Z",
            "name": "UFC Fight Night"
        }"#;

        let event: EventDto = serde_json::from_str(json).unwrap();
        let cloned = event.clone();

        assert_eq!(event.id, cloned.id);
        assert_eq!(event.name, cloned.name);
        assert_eq!(event.date, cloned.date);
    }
}
