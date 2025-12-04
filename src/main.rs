mod database;
mod engine;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let _connection = database::init();

    let espn = engine::espn::Espn::new();
    let result = espn.get_all_events(2024).await?;

    println!("{result:?}");

    Ok(())
}
