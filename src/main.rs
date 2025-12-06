mod database;
mod domain;
mod espn;

use database::Database;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let database = Database::new();

    let espn = espn::Espn::new();
    let result = espn.get_all_events(2024).await?;

    // println!("{result:?}");

    Ok(())
}
