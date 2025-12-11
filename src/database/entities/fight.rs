//! # Fight Entity
//!
//! Defines a struct which represents the entity for a `Fight`.

use time::OffsetDateTime;

#[derive(Debug, sqlx::FromRow)]
pub struct Fight {
    pub id: i64,
    pub event_id: i64,
    pub winner_id: i64,
    pub loser_id: i64,
    pub date: OffsetDateTime,
}
