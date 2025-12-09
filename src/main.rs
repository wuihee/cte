mod database;
mod domain;
mod engine;
mod espn;

use crate::{database::Database, engine::sync_fight_data};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let database = Database::new().await?;

    sync_fight_data(&database).await?;

    Ok(())
}
