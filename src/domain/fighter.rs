//! # Fighter
//!
//! Represents the data model for a fighter.

/// Represents the data for a UFC fighter.
#[derive(Debug, Clone)]
pub struct Fighter {
    /// The fighter's ID.
    pub id: i32,

    /// Name of the fighter.
    pub name: String,
}
