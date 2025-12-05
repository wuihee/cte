//! # Database
//!
//! This module contains the schema and setup for a database of UFC fighters,
//! fights, and ratings.

use rusqlite::{Connection, Result};

mod schema;

pub struct Database {
    connection: Connection,
}

impl Database {
    pub fn new() -> Result<Self> {
        let connection = Connection::open("data/app.db")?;
        schema::create_tables(&connection)?;
        Ok(Self { connection })
    }
}
