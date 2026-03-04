//! # Rating Entity
//!
//! Defines a struct which represents the entity for a `Rating`.

#[derive(Debug, sqlx::FromRow)]
#[allow(dead_code)]
pub struct Rating {
    pub id: i64,
    pub fighter_id: i64,
    pub fight_id: i64,
    pub rating: f64,
}
