mod database;
mod engine;
mod espn;
mod tui;

use clap::Parser;

use crate::{
    database::Database,
    engine::{SyncOptions, sync_fight_data, update_ratings},
    tui::run_app,
};

#[derive(Parser, Debug)]
#[command(name = "cte")]
#[command(about = "UFC Fighter Rankings with Enhanced Elo", long_about = None)]
struct Args {
    /// Sync fight data from ESPN
    #[arg(short, long)]
    sync: bool,

    /// Force re-sync all events (use with --sync)
    #[arg(short, long, requires = "sync")]
    force: bool,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    let database = Database::new().await?;

    if args.sync {
        let options = SyncOptions { force: args.force };

        println!("Syncing fight data from ESPN...\n");
        sync_fight_data(&database, &options).await?;

        println!("\nResetting ratings...");
        database.reset_ratings().await?;

        println!("Calculating ratings...");
        update_ratings(&database).await?;

        println!("Done! Run without --sync to view rankings.");
        return Ok(());
    }

    let fighters = database.get_fighters_by_rating().await?;
    if fighters.is_empty() {
        println!("No fighter data found. Run with --sync flag first:");
        println!("  cargo run -- --sync");
        return Ok(());
    }

    run_app(database).await?;

    Ok(())
}
