//! # Database
//!
//! Provides a unified entry point for accessing the application's database.

use std::env;

use sqlx::{Result, SqlitePool};

/// Encapsulates the applications main database connection via [`SqlitePool`].
pub struct Database {
    /// Will be passed to sqlx queries to perform database operations.
    pub pool: SqlitePool,
}

impl Database {
    /// The default database URL.
    const DEFAULT_URL: &'static str = "sqlite:data/app.db";

    /// Connects to the database and ensures migrations are up to date
    ///
    /// # Returns
    ///
    /// A [`Database`] instance ready to process queries.
    pub async fn new() -> Result<Self> {
        let url = env::var("DATABASE_URL").unwrap_or_else(|_| Self::DEFAULT_URL.to_string());

        let pool = SqlitePool::connect(&url).await?;
        sqlx::migrate!("./migrations").run(&pool).await?;

        Ok(Self { pool })
    }
}
