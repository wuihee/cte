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
    /// - `fight_time`: How long the fight took in seconds.
    /// - `weight_class`: Weight class the fight took place at.
    /// - `finish_method`: How the fight was finished.
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
        fight_time: u32,
        weight_class: &str,
        finish_method: &str,
    ) -> sqlx::Result<()> {
        sqlx::query!(
            r#"
            INSERT OR IGNORE INTO fights (
                id,
                event_id,
                winner_id,
                loser_id,
                date,
                fight_time,
                weight_class,
                finish_method
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            "#,
            id,
            event_id,
            winner_id,
            loser_id,
            date,
            fight_time,
            weight_class,
            finish_method,
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

    pub async fn insert_fight_stat(
        &self,
        fighter_id: &str,
        fight_id: &str,
        knock_downs: u32,
        total_strikes_hit: u32,
        total_strikes_missed: u32,
        sig_strikes: u32,
        head_strikes: u32,
        body_strikes: u32,
        leg_strikes: u32,
        time_in_control: u32,
        takedowns_hit: u32,
        takedowns_missed: u32,
        submissions_hit: u32,
        submissions_missed: u32,
    ) -> sqlx::Result<()> {
        sqlx::query!(
            r#"
            INSERT OR IGNORE INTO fight_stats (
                fighter_id,
                fight_id,
                knock_downs,
                total_strikes_hit,
                total_strikes_missed,
                sig_strikes,
                head_strikes,
                body_strikes,
                leg_strikes,
                time_in_control,
                takedowns_hit,
                takedowns_missed,
                submissions_hit,
                submissions_missed
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14)
            "#,
            fighter_id,
            fight_id,
            knock_downs,
            total_strikes_hit,
            total_strikes_missed,
            sig_strikes,
            head_strikes,
            body_strikes,
            leg_strikes,
            time_in_control,
            takedowns_hit,
            takedowns_missed,
            submissions_hit,
            submissions_missed,
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
            SELECT id, event_id, winner_id, loser_id, date, fight_time, weight_class, finish_method
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
            SELECT id, name, rating, max_rating, wins, losses, ko_wins, sub_wins, dec_wins, weight_class
            FROM fighters
            WHERE id = $1
            "#,
            id
        )
        .fetch_one(&self.pool)
        .await?;
        Ok(fighter)
    }

    /// Updates a fighter's record after a fight.
    ///
    /// # Arguments
    ///
    /// - `id`: Fighter's ID.
    /// - `rating`: The new rating value.
    /// - `max_rating`: The new max rating value.
    /// - `is_win`: Whether this was a win.
    /// - `finish_method`: How the fight ended (for win type tracking).
    /// - `weight_class`: The weight class of the fight.
    ///
    /// # Returns
    ///
    /// `Ok` if successfully updated, else `Err`.
    pub async fn update_fighter_after_fight(
        &self,
        id: i64,
        rating: f64,
        max_rating: f64,
        is_win: bool,
        finish_method: Option<&str>,
        weight_class: Option<&str>,
    ) -> sqlx::Result<()> {
        let fighter = self.get_fighter(id).await?;

        let (wins, losses, ko_wins, sub_wins, dec_wins) = if is_win {
            let method = finish_method.unwrap_or("");
            let is_ko = method.contains("KO") || method.contains("TKO");
            let is_sub = method.contains("SUB") || method.contains("Submission");
            (
                fighter.wins + 1,
                fighter.losses,
                fighter.ko_wins + if is_ko { 1 } else { 0 },
                fighter.sub_wins + if is_sub { 1 } else { 0 },
                fighter.dec_wins + if !is_ko && !is_sub { 1 } else { 0 },
            )
        } else {
            (
                fighter.wins,
                fighter.losses + 1,
                fighter.ko_wins,
                fighter.sub_wins,
                fighter.dec_wins,
            )
        };

        sqlx::query!(
            r#"
            UPDATE fighters
            SET rating = $1, max_rating = $2, wins = $3, losses = $4,
                ko_wins = $5, sub_wins = $6, dec_wins = $7, weight_class = COALESCE($8, weight_class)
            WHERE id = $9
            "#,
            rating,
            max_rating,
            wins,
            losses,
            ko_wins,
            sub_wins,
            dec_wins,
            weight_class,
            id,
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    /// Get all fighters ordered by rating (descending).
    ///
    /// # Returns
    ///
    /// A `Vec` of [`Fighter`] on success, else `Err`.
    pub async fn get_fighters_by_rating(&self) -> sqlx::Result<Vec<Fighter>> {
        let fighters = sqlx::query_as!(
            Fighter,
            r#"
            SELECT id, name, rating, max_rating, wins, losses, ko_wins, sub_wins, dec_wins, weight_class
            FROM fighters
            WHERE wins + losses > 0
            ORDER BY rating DESC
            "#
        )
        .fetch_all(&self.pool)
        .await?;
        Ok(fighters)
    }

    /// Get all fighters in a specific weight class ordered by rating.
    ///
    /// # Arguments
    ///
    /// - `weight_class`: The weight class slug.
    ///
    /// # Returns
    ///
    /// A `Vec` of [`Fighter`] on success, else `Err`.
    pub async fn get_fighters_by_weight_class(
        &self,
        weight_class: &str,
    ) -> sqlx::Result<Vec<Fighter>> {
        let fighters = sqlx::query_as!(
            Fighter,
            r#"
            SELECT id, name, rating, max_rating, wins, losses, ko_wins, sub_wins, dec_wins, weight_class
            FROM fighters
            WHERE weight_class = $1 AND wins + losses > 0
            ORDER BY rating DESC
            "#,
            weight_class
        )
        .fetch_all(&self.pool)
        .await?;
        Ok(fighters)
    }

    /// Get fight history for a specific fighter.
    ///
    /// # Arguments
    ///
    /// - `fighter_id`: Fighter's ID.
    ///
    /// # Returns
    ///
    /// A `Vec` of [`Fight`] on success, else `Err`.
    pub async fn get_fighter_fights(&self, fighter_id: i64) -> sqlx::Result<Vec<Fight>> {
        let fights = sqlx::query_as!(
            Fight,
            r#"
            SELECT id, event_id, winner_id, loser_id, date, fight_time, weight_class, finish_method
            FROM fights
            WHERE winner_id = $1 OR loser_id = $1
            ORDER BY date DESC
            "#,
            fighter_id
        )
        .fetch_all(&self.pool)
        .await?;
        Ok(fights)
    }

    /// Get a fighter's name by ID.
    pub async fn get_fighter_name(&self, id: i64) -> sqlx::Result<String> {
        let name = sqlx::query_scalar!(r#"SELECT name FROM fighters WHERE id = $1"#, id)
            .fetch_one(&self.pool)
            .await?;
        Ok(name)
    }

    /// Check if an event has already been synced.
    pub async fn is_event_synced(&self, event_id: &str) -> sqlx::Result<bool> {
        let count = sqlx::query_scalar!(
            r#"SELECT COUNT(*) as count FROM sync_log WHERE event_id = $1"#,
            event_id
        )
        .fetch_one(&self.pool)
        .await?;
        Ok(count > 0)
    }

    /// Mark an event as synced.
    pub async fn mark_event_synced(
        &self,
        event_id: &str,
        event_name: &str,
        fights_count: i32,
    ) -> sqlx::Result<()> {
        sqlx::query!(
            r#"
            INSERT OR REPLACE INTO sync_log (event_id, event_name, fights_count, synced_at)
            VALUES ($1, $2, $3, CURRENT_TIMESTAMP)
            "#,
            event_id,
            event_name,
            fights_count,
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    /// Get sync statistics: (events_synced, fights_synced).
    pub async fn get_sync_stats(&self) -> sqlx::Result<(i64, i64)> {
        let stats = sqlx::query!(
            r#"
            SELECT COUNT(*) as events, COALESCE(SUM(fights_count), 0) as fights
            FROM sync_log
            "#
        )
        .fetch_one(&self.pool)
        .await?;
        Ok((stats.events, stats.fights))
    }

    /// Clear the sync log.
    pub async fn clear_sync_log(&self) -> sqlx::Result<()> {
        sqlx::query!("DELETE FROM sync_log")
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    /// Reset all fighter ratings to starting values.
    /// Clears rating history to prevent compounding.
    pub async fn reset_ratings(&self) -> sqlx::Result<()> {
        sqlx::query!(
            r#"
            UPDATE fighters SET
                rating = 1000,
                max_rating = 1000,
                wins = 0,
                losses = 0,
                ko_wins = 0,
                sub_wins = 0,
                dec_wins = 0
            "#
        )
        .execute(&self.pool)
        .await?;

        sqlx::query!("DELETE FROM ratings")
            .execute(&self.pool)
            .await?;

        Ok(())
    }
}
