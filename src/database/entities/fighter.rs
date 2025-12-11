//! # Fighter Entity
//!
//! Defines a struct which represents the entity for a `Fighter`.

#[derive(Debug, sqlx::FromRow)]
pub struct Fighter {
    pub id: i64,
    pub name: String,
    pub rating: f64,
    pub max_rating: f64,
}
