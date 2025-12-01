//! # Schema
//!
//! This module defines the database schema.

use rusqlite::{Connection, Result};

/// Creates the tables in the database.
///
/// # Arguments
///
/// - `connection`: A sqlite connection.
///
/// # Returns
pub fn create_tables(connection: &Connection) -> Result<()> {
    connection.execute(
        "
        CREATE TABLE IF NOT EXISTS fighters (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL
        );
        ",
        [],
    )?;

    Ok(())
}
