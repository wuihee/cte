mod database;
mod engine;
mod espn;

use crate::{
    database::Database,
    engine::{sync_fight_data, update_ratings},
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let database = Database::new().await?;

    println!("Syncing fight data...");
    sync_fight_data(&database).await?;

    println!("Calculating ratings...");
    update_ratings(&database).await?;

    Ok(())
}
