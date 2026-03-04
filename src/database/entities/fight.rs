//! # Fight Entity
//!
//! Defines a struct which represents the entity for a `Fight`.

use time::OffsetDateTime;

#[derive(Debug, Clone, sqlx::FromRow)]
#[allow(dead_code)]
pub struct Fight {
    pub id: i64,
    pub event_id: i64,
    pub winner_id: i64,
    pub loser_id: i64,
    pub date: OffsetDateTime,
    pub fight_time: i64,
    pub weight_class: Option<String>,
    pub finish_method: Option<String>,
}
