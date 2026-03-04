//! Integration tests for database operations.
//!
//! These tests use an in-memory SQLite database to test database operations
//! without affecting the actual database.

use sqlx::SqlitePool;

/// Creates an in-memory database with the schema applied.
async fn setup_test_db() -> SqlitePool {
    let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();

    // Apply schema
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS events (
            id INTEGER PRIMARY KEY,
            name TEXT NOT NULL,
            date DATETIME NOT NULL,
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP
        );

        CREATE TABLE IF NOT EXISTS fighters (
            id INTEGER PRIMARY KEY,
            name TEXT NOT NULL,
            rating REAL NOT NULL DEFAULT 1000,
            max_rating REAL NOT NULL DEFAULT 1000,
            wins INTEGER NOT NULL DEFAULT 0,
            losses INTEGER NOT NULL DEFAULT 0,
            ko_wins INTEGER NOT NULL DEFAULT 0,
            sub_wins INTEGER NOT NULL DEFAULT 0,
            dec_wins INTEGER NOT NULL DEFAULT 0,
            weight_class TEXT,
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP
        );

        CREATE TABLE IF NOT EXISTS fights (
            id INTEGER PRIMARY KEY,
            event_id INTEGER NOT NULL,
            winner_id INTEGER NOT NULL,
            loser_id INTEGER NOT NULL,
            date DATETIME NOT NULL,
            fight_time INTEGER NOT NULL,
            weight_class TEXT,
            finish_method TEXT,
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
            FOREIGN KEY (event_id) REFERENCES events(id) ON DELETE CASCADE,
            FOREIGN KEY (winner_id) REFERENCES fighters(id) ON DELETE CASCADE,
            FOREIGN KEY (loser_id) REFERENCES fighters(id) ON DELETE CASCADE
        );

        CREATE TABLE IF NOT EXISTS ratings (
            id INTEGER PRIMARY KEY,
            fighter_id INTEGER NOT NULL,
            fight_id INTEGER NOT NULL,
            rating REAL NOT NULL,
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
            FOREIGN KEY (fighter_id) REFERENCES fighters(id),
            FOREIGN KEY (fight_id) REFERENCES fights(id),
            UNIQUE (fighter_id, fight_id)
        );

        CREATE TABLE IF NOT EXISTS fight_stats (
            id INTEGER PRIMARY KEY,
            fighter_id INTEGER NOT NULL,
            fight_id INTEGER NOT NULL,
            knock_downs INTEGER,
            total_strikes_hit INTEGER,
            total_strikes_missed INTEGER,
            sig_strikes INTEGER,
            head_strikes INTEGER,
            body_strikes INTEGER,
            leg_strikes INTEGER,
            time_in_control INTEGER,
            takedowns_hit INTEGER,
            takedowns_missed INTEGER,
            submissions_hit INTEGER,
            submissions_missed INTEGER,
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
            FOREIGN KEY (fighter_id) REFERENCES fighters(id),
            FOREIGN KEY (fight_id) REFERENCES fights(id),
            UNIQUE (fighter_id, fight_id)
        );
        "#,
    )
    .execute(&pool)
    .await
    .unwrap();

    pool
}

// ==================== Fighter Tests ====================

#[tokio::test]
async fn test_insert_and_get_fighter() {
    let pool = setup_test_db().await;

    // Insert a fighter
    sqlx::query("INSERT INTO fighters (id, name) VALUES (1, 'Jon Jones')")
        .execute(&pool)
        .await
        .unwrap();

    // Retrieve the fighter
    let fighter: (i64, String, f64) =
        sqlx::query_as("SELECT id, name, rating FROM fighters WHERE id = 1")
            .fetch_one(&pool)
            .await
            .unwrap();

    assert_eq!(fighter.0, 1);
    assert_eq!(fighter.1, "Jon Jones");
    assert_eq!(fighter.2, 1000.0); // Default rating
}

#[tokio::test]
async fn test_fighter_default_values() {
    let pool = setup_test_db().await;

    sqlx::query("INSERT INTO fighters (id, name) VALUES (1, 'Test Fighter')")
        .execute(&pool)
        .await
        .unwrap();

    let fighter: (f64, f64, i64, i64, i64, i64, i64) = sqlx::query_as(
        "SELECT rating, max_rating, wins, losses, ko_wins, sub_wins, dec_wins FROM fighters WHERE id = 1",
    )
    .fetch_one(&pool)
    .await
    .unwrap();

    assert_eq!(fighter.0, 1000.0); // rating
    assert_eq!(fighter.1, 1000.0); // max_rating
    assert_eq!(fighter.2, 0); // wins
    assert_eq!(fighter.3, 0); // losses
    assert_eq!(fighter.4, 0); // ko_wins
    assert_eq!(fighter.5, 0); // sub_wins
    assert_eq!(fighter.6, 0); // dec_wins
}

#[tokio::test]
async fn test_update_fighter_rating() {
    let pool = setup_test_db().await;

    sqlx::query("INSERT INTO fighters (id, name) VALUES (1, 'Test Fighter')")
        .execute(&pool)
        .await
        .unwrap();

    // Update rating
    sqlx::query("UPDATE fighters SET rating = 1200.0, max_rating = 1200.0 WHERE id = 1")
        .execute(&pool)
        .await
        .unwrap();

    let rating: (f64,) = sqlx::query_as("SELECT rating FROM fighters WHERE id = 1")
        .fetch_one(&pool)
        .await
        .unwrap();

    assert_eq!(rating.0, 1200.0);
}

#[tokio::test]
async fn test_insert_duplicate_fighter_ignored() {
    let pool = setup_test_db().await;

    sqlx::query("INSERT INTO fighters (id, name) VALUES (1, 'Fighter A')")
        .execute(&pool)
        .await
        .unwrap();

    // Try to insert duplicate - should be ignored with INSERT OR IGNORE
    let result = sqlx::query("INSERT OR IGNORE INTO fighters (id, name) VALUES (1, 'Fighter B')")
        .execute(&pool)
        .await
        .unwrap();

    // Row was ignored (0 rows affected)
    assert_eq!(result.rows_affected(), 0);

    // Original name should be preserved
    let name: (String,) = sqlx::query_as("SELECT name FROM fighters WHERE id = 1")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(name.0, "Fighter A");
}

// ==================== Fight Tests ====================

#[tokio::test]
async fn test_insert_fight() {
    let pool = setup_test_db().await;

    // Setup: Insert event and fighters first
    sqlx::query("INSERT INTO events (id, name, date) VALUES (1, 'UFC 300', '2024-04-13')")
        .execute(&pool)
        .await
        .unwrap();

    sqlx::query("INSERT INTO fighters (id, name) VALUES (1, 'Fighter A'), (2, 'Fighter B')")
        .execute(&pool)
        .await
        .unwrap();

    // Insert fight
    sqlx::query(
        "INSERT INTO fights (id, event_id, winner_id, loser_id, date, fight_time, weight_class, finish_method)
         VALUES (1, 1, 1, 2, '2024-04-13', 300, 'lightweight', 'KO/TKO')",
    )
    .execute(&pool)
    .await
    .unwrap();

    let fight: (i64, i64, i64, String) =
        sqlx::query_as("SELECT id, winner_id, loser_id, finish_method FROM fights WHERE id = 1")
            .fetch_one(&pool)
            .await
            .unwrap();

    assert_eq!(fight.0, 1);
    assert_eq!(fight.1, 1); // winner_id
    assert_eq!(fight.2, 2); // loser_id
    assert_eq!(fight.3, "KO/TKO");
}

#[tokio::test]
async fn test_fights_ordered_by_date() {
    let pool = setup_test_db().await;

    // Setup
    sqlx::query("INSERT INTO events (id, name, date) VALUES (1, 'Event 1', '2024-01-01'), (2, 'Event 2', '2024-06-01')")
        .execute(&pool)
        .await
        .unwrap();

    sqlx::query("INSERT INTO fighters (id, name) VALUES (1, 'A'), (2, 'B'), (3, 'C'), (4, 'D')")
        .execute(&pool)
        .await
        .unwrap();

    // Insert fights out of order
    sqlx::query(
        "INSERT INTO fights (id, event_id, winner_id, loser_id, date, fight_time) VALUES
         (2, 2, 3, 4, '2024-06-01', 300),
         (1, 1, 1, 2, '2024-01-01', 300)",
    )
    .execute(&pool)
    .await
    .unwrap();

    // Query ordered by date
    let fights: Vec<(i64,)> = sqlx::query_as("SELECT id FROM fights ORDER BY date ASC")
        .fetch_all(&pool)
        .await
        .unwrap();

    assert_eq!(fights.len(), 2);
    assert_eq!(fights[0].0, 1); // Earlier fight first
    assert_eq!(fights[1].0, 2);
}

// ==================== Rating Tests ====================

#[tokio::test]
async fn test_insert_rating() {
    let pool = setup_test_db().await;

    // Setup
    sqlx::query("INSERT INTO events (id, name, date) VALUES (1, 'Event', '2024-01-01')")
        .execute(&pool)
        .await
        .unwrap();
    sqlx::query("INSERT INTO fighters (id, name) VALUES (1, 'Fighter')")
        .execute(&pool)
        .await
        .unwrap();
    sqlx::query(
        "INSERT INTO fights (id, event_id, winner_id, loser_id, date, fight_time) VALUES (1, 1, 1, 1, '2024-01-01', 300)",
    )
    .execute(&pool)
    .await
    .unwrap();

    // Insert rating
    sqlx::query("INSERT INTO ratings (fighter_id, fight_id, rating) VALUES (1, 1, 1050.0)")
        .execute(&pool)
        .await
        .unwrap();

    let rating: (f64,) =
        sqlx::query_as("SELECT rating FROM ratings WHERE fighter_id = 1 AND fight_id = 1")
            .fetch_one(&pool)
            .await
            .unwrap();

    assert_eq!(rating.0, 1050.0);
}

#[tokio::test]
async fn test_rating_unique_constraint() {
    let pool = setup_test_db().await;

    // Setup
    sqlx::query("INSERT INTO events (id, name, date) VALUES (1, 'Event', '2024-01-01')")
        .execute(&pool)
        .await
        .unwrap();
    sqlx::query("INSERT INTO fighters (id, name) VALUES (1, 'Fighter')")
        .execute(&pool)
        .await
        .unwrap();
    sqlx::query(
        "INSERT INTO fights (id, event_id, winner_id, loser_id, date, fight_time) VALUES (1, 1, 1, 1, '2024-01-01', 300)",
    )
    .execute(&pool)
    .await
    .unwrap();

    // Insert first rating
    sqlx::query("INSERT INTO ratings (fighter_id, fight_id, rating) VALUES (1, 1, 1050.0)")
        .execute(&pool)
        .await
        .unwrap();

    // Try to insert duplicate - should be ignored
    let result = sqlx::query(
        "INSERT OR IGNORE INTO ratings (fighter_id, fight_id, rating) VALUES (1, 1, 1100.0)",
    )
    .execute(&pool)
    .await
    .unwrap();

    assert_eq!(result.rows_affected(), 0);

    // Original rating preserved
    let rating: (f64,) =
        sqlx::query_as("SELECT rating FROM ratings WHERE fighter_id = 1 AND fight_id = 1")
            .fetch_one(&pool)
            .await
            .unwrap();

    assert_eq!(rating.0, 1050.0);
}

// ==================== Fight Stats Tests ====================

#[tokio::test]
async fn test_insert_fight_stats() {
    let pool = setup_test_db().await;

    // Setup
    sqlx::query("INSERT INTO events (id, name, date) VALUES (1, 'Event', '2024-01-01')")
        .execute(&pool)
        .await
        .unwrap();
    sqlx::query("INSERT INTO fighters (id, name) VALUES (1, 'Fighter')")
        .execute(&pool)
        .await
        .unwrap();
    sqlx::query(
        "INSERT INTO fights (id, event_id, winner_id, loser_id, date, fight_time) VALUES (1, 1, 1, 1, '2024-01-01', 300)",
    )
    .execute(&pool)
    .await
    .unwrap();

    // Insert stats
    sqlx::query(
        "INSERT INTO fight_stats (fighter_id, fight_id, knock_downs, sig_strikes, takedowns_hit, time_in_control)
         VALUES (1, 1, 2, 45, 3, 120)",
    )
    .execute(&pool)
    .await
    .unwrap();

    let stats: (i64, i64, i64, i64) = sqlx::query_as(
        "SELECT knock_downs, sig_strikes, takedowns_hit, time_in_control FROM fight_stats WHERE fighter_id = 1",
    )
    .fetch_one(&pool)
    .await
    .unwrap();

    assert_eq!(stats.0, 2); // knock_downs
    assert_eq!(stats.1, 45); // sig_strikes
    assert_eq!(stats.2, 3); // takedowns_hit
    assert_eq!(stats.3, 120); // time_in_control
}

// ==================== Ranking Query Tests ====================

#[tokio::test]
async fn test_get_fighters_ordered_by_rating() {
    let pool = setup_test_db().await;

    // Insert fighters with different ratings
    sqlx::query(
        "INSERT INTO fighters (id, name, rating, wins, losses) VALUES
         (1, 'Champion', 1400.0, 10, 0),
         (2, 'Contender', 1200.0, 8, 2),
         (3, 'Newcomer', 1000.0, 1, 0)",
    )
    .execute(&pool)
    .await
    .unwrap();

    let fighters: Vec<(String, f64)> = sqlx::query_as(
        "SELECT name, rating FROM fighters WHERE wins + losses > 0 ORDER BY rating DESC",
    )
    .fetch_all(&pool)
    .await
    .unwrap();

    assert_eq!(fighters.len(), 3);
    assert_eq!(fighters[0].0, "Champion");
    assert_eq!(fighters[1].0, "Contender");
    assert_eq!(fighters[2].0, "Newcomer");
}

#[tokio::test]
async fn test_get_fighters_by_weight_class() {
    let pool = setup_test_db().await;

    sqlx::query(
        "INSERT INTO fighters (id, name, rating, wins, losses, weight_class) VALUES
         (1, 'LW Champ', 1400.0, 10, 0, 'lightweight'),
         (2, 'LW Contender', 1200.0, 8, 2, 'lightweight'),
         (3, 'HW Fighter', 1300.0, 5, 1, 'heavyweight')",
    )
    .execute(&pool)
    .await
    .unwrap();

    let lightweights: Vec<(String,)> = sqlx::query_as(
        "SELECT name FROM fighters WHERE weight_class = 'lightweight' AND wins + losses > 0 ORDER BY rating DESC",
    )
    .fetch_all(&pool)
    .await
    .unwrap();

    assert_eq!(lightweights.len(), 2);
    assert_eq!(lightweights[0].0, "LW Champ");
    assert_eq!(lightweights[1].0, "LW Contender");
}

// ==================== Foreign Key Tests ====================

#[tokio::test]
async fn test_fight_references_valid_fighters() {
    let pool = setup_test_db().await;

    // Enable foreign keys
    sqlx::query("PRAGMA foreign_keys = ON")
        .execute(&pool)
        .await
        .unwrap();

    sqlx::query("INSERT INTO events (id, name, date) VALUES (1, 'Event', '2024-01-01')")
        .execute(&pool)
        .await
        .unwrap();

    sqlx::query("INSERT INTO fighters (id, name) VALUES (1, 'A'), (2, 'B')")
        .execute(&pool)
        .await
        .unwrap();

    // This should succeed - valid fighter IDs
    let result = sqlx::query(
        "INSERT INTO fights (id, event_id, winner_id, loser_id, date, fight_time) VALUES (1, 1, 1, 2, '2024-01-01', 300)",
    )
    .execute(&pool)
    .await;

    assert!(result.is_ok());
}
