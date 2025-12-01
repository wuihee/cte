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
///
/// `Ok` on success or `Err` on failure.
pub fn create_tables(connection: &Connection) -> Result<()> {
    create_fighters_table(connection)?;
    create_fights_table(connection)?;
    create_ratings_table(connection)?;
    Ok(())
}

/// Creates the table for UFC fighters.
fn create_fighters_table(connection: &Connection) -> Result<()> {
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

/// Creates the table for UFC fights.
fn create_fights_table(connection: &Connection) -> Result<()> {
    connection.execute(
        "
        CREATE TABLE IF NOT EXISTS fights (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            fighter_1 INTEGER,
            fighter_2 INTEGER,
            winner INTEGER,
            loser INTEGER,
            fight_date TEXT,

            FOREIGN KEY (fighter_1) REFERENCES fighters(id),
            FOREIGN KEY (fighter_2) REFERENCES fighters(id)
        );
        ",
        [],
    )?;
    Ok(())
}

/// Creates the table for the ratings.
///
/// After every fight, the ratings of both fighters will change. This table
/// will capture the chronological change in ratings for fighters.
fn create_ratings_table(connection: &Connection) -> Result<()> {
    connection.execute(
        "
        CREATE TABLE IF NOT EXISTS ratings (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            fighter_id INTEGER NOT NULL,
            fight_id INTEGER NOT NULL,
            rating REAL NOT NULL,

            FOREIGN KEY (fighter_id) REFERENCES fighters(id),
            FOREIGN KEY (fight_id) REFERENCES fights(id)
        );
        ",
        [],
    )?;

    Ok(())
}
