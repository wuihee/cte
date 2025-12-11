//! # Database
//!
//! Provides a unified entry point for accessing the application's database.

pub mod entities;

use std::env;

use sqlx::SqlitePool;
use time::OffsetDateTime;

use crate::database::entities::{Fight, Fighter};

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
    pub async fn new() -> sqlx::Result<Self> {
        let url = env::var("DATABASE_URL").unwrap_or_else(|_| Self::DEFAULT_URL.to_string());
        let pool = SqlitePool::connect(&url).await?;

        // Enable support for foreign keys in sqlite.
        sqlx::query("PRAGMA foreign_keys = ON;")
            .execute(&pool)
            .await?;
        sqlx::migrate!("./migrations").run(&pool).await?;

        Ok(Self { pool })
    }

    /// Insert a UFC event into the database.
    ///
    /// Skip if event already exists.
    ///
    /// # Arguments
    ///
    /// - `id`: The event ID; provided by ESPN;
    /// - `name`: Event name. E.g. "UFC 223".
    /// - `date`: Date of the event.
    ///
    /// # Returns
    ///
    /// `Ok` if successfully inserted, else `Err`.
    pub async fn insert_event(
        &self,
        id: &str,
        name: &str,
        date: &OffsetDateTime,
    ) -> sqlx::Result<()> {
        sqlx::query!(
            r#"
            INSERT OR IGNORE INTO events (id, name, date)
            VALUES ($1, $2, $3)
            "#,
            id,
            name,
            date
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    /// Insert a UFC fighter into the database.
    ///
    /// Skip if fighter already exists.
    ///
    /// # Arguments
    ///
    /// - `id`: The fighter ID; provided by ESPN.
    /// - `name`: Fighter name.
    ///
    /// # Returns
    ///
    /// `Ok` if successfully inserted, else `Err`.
    pub async fn insert_fighter(&self, id: &str, name: &str) -> sqlx::Result<()> {
        sqlx::query!(
            r#"
            INSERT OR IGNORE INTO fighters (id, name)
            VALUES ($1, $2)
            "#,
            id,
            name
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    /// Insert a UFC fight between two fighters into the database.
    ///
    /// Skip if fight already exists.
    ///
    /// # Arguments
    ///
    /// - `id`: The fight; provided by ESPN as the ID of a `competition`.
    /// - `event_id`: The ID for the event which the fight took place on;
    ///               provided by ESPN.
    /// - `winner_id`: The fighter ID of the winner.
    /// - `loser_id`: The fighter ID of the loser.
    ///
    /// # Returns
    ///
    /// `Ok` if successfully inserted, else `Err`.
    pub async fn insert_fight(
        &self,
        id: &str,
        event_id: &str,
        winner_id: &str,
        loser_id: &str,
        date: &OffsetDateTime,
    ) -> sqlx::Result<()> {
        sqlx::query!(
            r#"
            INSERT OR IGNORE INTO fights (id, event_id, winner_id, loser_id, date)
            VALUES ($1, $2, $3, $4, $5)
            "#,
            id,
            event_id,
            winner_id,
            loser_id,
            date,
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    /// Insert a rating for a UFC fighter after a certain fight.
    ///
    /// Skip if rating already exists.
    ///
    /// # Arguments
    ///
    /// - `fighter_id`: Fighter's ID.
    /// - `fight_id`: Fight's ID.
    /// - `rating`: The rating of the fighter after the fight.
    ///
    /// # Returns
    ///
    /// `Ok` if successfully inserted, else `Err`.
    pub async fn insert_rating(
        &self,
        fighter_id: i64,
        fight_id: i64,
        rating: f64,
    ) -> sqlx::Result<()> {
        sqlx::query!(
            r#"
            INSERT OR IGNORE INTO ratings (fighter_id, fight_id, rating)
            VALUES ($1, $2, $3)
            "#,
            fighter_id,
            fight_id,
            rating,
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    /// Get all fights ordered by date in ascending order.
    ///
    /// # Returns
    ///
    /// A `Vec` of [`Fight`] on success, else `Err`.
    pub async fn get_fights_order_by_date(&self) -> sqlx::Result<Vec<Fight>> {
        let fight = sqlx::query_as!(
            Fight,
            r#"
            SELECT id, event_id, winner_id, loser_id, date
            FROM fights
            ORDER BY date ASC
            "#
        )
        .fetch_all(&self.pool)
        .await?;
        Ok(fight)
    }

    /// Get a fighter by their ID.
    ///
    /// # Arguments
    ///
    /// - `id`: Fighter's ID.
    ///
    /// # Returns
    ///
    /// `Ok(Fighter)` on success, else `Err`.
    pub async fn get_fighter(&self, id: i64) -> sqlx::Result<Fighter> {
        let fighter = sqlx::query_as!(
            Fighter,
            r#"
            SELECT id, name, rating, max_rating FROM fighters
            WHERE id = $1
            "#,
            id
        )
        .fetch_one(&self.pool)
        .await?;
        Ok(fighter)
    }

    pub async fn update_figher_rating(
        &self,
        id: i64,
        rating: f64,
        max_rating: f64,
    ) -> sqlx::Result<()> {
        sqlx::query!(
            r#"
            UPDATE fighters
            SET rating = $1, max_rating = $2
            WHERE id = $3
            "#,
            rating,
            max_rating,
            id,
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }
}
