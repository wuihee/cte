//! # Rating Entity
//!
//! Defines a struct which represents the entity for a `Rating`.

#[derive(Debug, sqlx::FromRow)]
pub struct Fighter {
    pub id: i64,
    pub fighter_id: String,
    pub fight_id: f64,
    pub rating: f64,
}
