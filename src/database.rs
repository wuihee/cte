//! # Database
//!
//! Provides a unified entry point for accessing the application's database.

use sqlx::{Result, SqlitePool};

/// Encapsulates the applications main database connection via [`SqlitePool`].
pub struct Database {
    /// Will be passed to sqlx queries to perform database operations.
    pub pool: SqlitePool,
}

impl Database {
    const DATABASE_URL: &'static str = "sqlite:data/app.db";

    /// Connects to the database and ensures migrations are up to date
    ///
    /// # Returns
    ///
    /// A [`Database`] instance ready to process queries.
    pub async fn new() -> Result<Self> {
        let pool = SqlitePool::connect(Database::DATABASE_URL).await?;
        sqlx::migrate!("./migrations").run(&pool).await?;
        Ok(Self { pool })
    }
}
