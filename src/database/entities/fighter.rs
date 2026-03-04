//! # Fighter Entity
//!
//! Defines a struct which represents the entity for a `Fighter`.

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct Fighter {
    pub id: i64,
    pub name: String,
    pub rating: f64,
    pub max_rating: f64,
    pub wins: i64,
    pub losses: i64,
    pub ko_wins: i64,
    pub sub_wins: i64,
    pub dec_wins: i64,
    pub weight_class: Option<String>,
}
