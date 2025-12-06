//! # Ratings
//!
//! The data model for the rating for a fighter at a point in time.

/// Represents a fighter's rating after a fight.
#[derive(Debug, Clone)]
pub struct Rating {
    /// The fighter's ID.
    pub fighter_id: i32,

    /// The fight's ID.
    pub fight_id: i32,

    /// The rating of the fighter after the fight.
    pub rating: f64,
}
