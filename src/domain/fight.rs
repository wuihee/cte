//! # Fight
//!
//! The data model for a single UFC fight.

/// Represents the outcome of a UFC fight.
#[derive(Debug, Clone)]
pub struct Fight {
    /// The fight ID.
    pub id: i32,

    /// Event which the fight was on.
    pub event_id: i32,

    /// ID of the fighter who won.
    pub winner_id: i32,

    /// ID of the fighter who lost.
    pub loser_id: i32,
}
