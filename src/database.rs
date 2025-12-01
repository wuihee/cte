//! # Database
//!
//! This module contains the schema and setup for a database of UFC fighters,
//! fights, and ratings.

use rusqlite::{Connection, Result};

mod schema;

/// Initialize the database and creates all tables.
///
/// # Returns
///
/// An active connection to the database.
pub fn init() -> Result<Connection> {
    let connection = Connection::open("data/app.db")?;
    schema::create_tables(&connection)?;
    Ok(connection)
}
