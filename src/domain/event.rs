//! # Event
//!
//! The data model for a single UFC event.

/// Represents a UFC fight event which holds many fights.
#[derive(Debug, Clone)]
pub struct Event {
    /// The ID of the event.
    pub id: i32,

    /// Name of the event.
    pub name: String,

    /// Date of the event.
    pub date: String,
}
