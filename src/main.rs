mod database;
mod engine;
mod espn;
mod tui;

use std::env;

use crate::{
    database::Database,
    engine::{SyncOptions, sync_fight_data, update_ratings},
    tui::run_app,
};

fn print_usage() {
    println!("Combat Training Engine - UFC Fighter Rankings\n");
    println!("USAGE:");
    println!("    cargo run [OPTIONS]\n");
    println!("OPTIONS:");
    println!("    --sync       Sync fight data from ESPN");
    println!("    --force      Force re-sync all events (use with --sync)");
    println!("    --help       Show this help message\n");
    println!("EXAMPLES:");
    println!("    cargo run                    # Launch TUI");
    println!("    cargo run -- --sync          # Sync new events only");
    println!("    cargo run -- --sync --force  # Re-sync all events");
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args: Vec<String> = env::args().collect();

    // Check for --help flag
    if args.contains(&"--help".to_string()) || args.contains(&"-h".to_string()) {
        print_usage();
        return Ok(());
    }

    let database = Database::new().await?;

    // Check for --sync flag to sync data
    if args.contains(&"--sync".to_string()) {
        let options = SyncOptions {
            force: args.contains(&"--force".to_string()),
        };

        println!("Syncing fight data from ESPN...\n");
        sync_fight_data(&database, &options).await?;

        println!("\nCalculating ratings...");
        update_ratings(&database).await?;

        println!("Done! Run without --sync to view rankings.");
        return Ok(());
    }

    // Check if we have any fighters
    let fighters = database.get_fighters_by_rating().await?;
    if fighters.is_empty() {
        println!("No fighter data found. Run with --sync flag first:");
        println!("  cargo run -- --sync");
        return Ok(());
    }

    // Run the TUI
    run_app(database).await?;

    Ok(())
}
