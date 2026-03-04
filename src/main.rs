mod database;
mod engine;
mod espn;
mod tui;

use clap::{Parser, Subcommand};

use crate::{
    database::Database,
    engine::{
        EloConfig, Optimizer, ParameterRanges, SyncOptions, export_results_to_csv,
        print_top_results, sync_fight_data, update_ratings,
    },
    tui::run_app,
};

#[derive(Parser, Debug)]
#[command(name = "cte")]
#[command(about = "UFC Fighter Rankings with Enhanced Elo", long_about = None)]
struct Args {
    #[command(subcommand)]
    command: Option<Command>,

    /// Sync fight data from ESPN (legacy flag, use 'sync' subcommand instead)
    #[arg(short, long)]
    sync: bool,

    /// Force re-sync all events (use with --sync)
    #[arg(short, long, requires = "sync")]
    force: bool,
}

#[derive(Subcommand, Debug)]
enum Command {
    /// Sync fight data from ESPN
    Sync {
        /// Force re-sync all events
        #[arg(short, long)]
        force: bool,
    },

    /// Optimize Elo parameters using historical data
    Optimize {
        /// Search method: 'grid' or 'random'
        #[arg(short, long, default_value = "grid")]
        method: String,

        /// Number of random samples (only for random search)
        #[arg(short, long, default_value = "50")]
        samples: usize,

        /// Random seed for reproducibility (only for random search)
        #[arg(long, default_value = "42")]
        seed: u64,

        /// Export results to CSV file
        #[arg(short, long)]
        export: Option<String>,

        /// Number of top results to display
        #[arg(short, long, default_value = "10")]
        top: usize,
    },

    /// Run a backtest with specific parameters
    Backtest {
        /// K-factor value
        #[arg(long, default_value = "32.0")]
        k_factor: f64,

        /// Finish multiplier
        #[arg(long, default_value = "1.0")]
        finish: f64,

        /// Title fight multiplier
        #[arg(long, default_value = "1.0")]
        title: f64,

        /// Five-round fight multiplier
        #[arg(long, default_value = "1.0")]
        five_round: f64,
    },
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    let database = Database::new().await?;

    // Handle subcommands
    if let Some(command) = args.command {
        match command {
            Command::Sync { force } => {
                run_sync(&database, force).await?;
            }
            Command::Optimize {
                method,
                samples,
                seed,
                export,
                top,
            } => {
                run_optimize(&database, &method, samples, seed, export, top).await?;
            }
            Command::Backtest {
                k_factor,
                finish,
                title,
                five_round,
            } => {
                run_backtest(&database, k_factor, finish, title, five_round).await?;
            }
        }
        return Ok(());
    }

    // Handle legacy --sync flag
    if args.sync {
        run_sync(&database, args.force).await?;
        return Ok(());
    }

    // Default: run TUI
    let fighters = database.get_fighters_by_rating().await?;
    if fighters.is_empty() {
        println!("No fighter data found. Run with --sync flag first:");
        println!("  cargo run -- --sync");
        println!("\nOr use the sync subcommand:");
        println!("  cargo run -- sync");
        return Ok(());
    }

    run_app(database).await?;

    Ok(())
}

/// Runs the sync operation.
async fn run_sync(database: &Database, force: bool) -> anyhow::Result<()> {
    let options = SyncOptions { force };

    println!("Syncing fight data from ESPN...\n");
    sync_fight_data(database, &options).await?;

    println!("\nResetting ratings...");
    database.reset_ratings().await?;

    println!("Calculating ratings...");
    update_ratings(database, &EloConfig::default()).await?;

    println!("Done! Run without --sync to view rankings.");
    Ok(())
}

/// Runs parameter optimization.
async fn run_optimize(
    database: &Database,
    method: &str,
    samples: usize,
    seed: u64,
    export: Option<String>,
    top: usize,
) -> anyhow::Result<()> {
    println!("Loading fight data...");
    let fights = database.get_fights_order_by_date().await?;

    if fights.is_empty() {
        println!("No fight data found. Run 'sync' first:");
        println!("  cargo run -- sync");
        return Ok(());
    }

    println!("Found {} fights for optimization.\n", fights.len());

    let ranges = ParameterRanges::default();
    let optimizer =
        Optimizer::with_ranges(ranges.clone()).with_progress(|current, total, config| {
            print!(
                "\rProgress: {}/{} ({:.1}%) - Testing K={:.0}, F={:.2}, T={:.2}, 5R={:.2}    ",
                current,
                total,
                (current as f64 / total as f64) * 100.0,
                config.k_factor,
                config.finish_multiplier,
                config.title_fight_multiplier,
                config.five_round_multiplier,
            );
            std::io::Write::flush(&mut std::io::stdout()).ok();
        });

    let (best, results) = match method.to_lowercase().as_str() {
        "grid" => {
            println!(
                "Running grid search over {} configurations...\n",
                optimizer.total_configurations()
            );
            optimizer.grid_search(&fights)
        }
        "random" => {
            println!(
                "Running random search with {} samples (seed={})...\n",
                samples, seed
            );
            optimizer.random_search(&fights, samples, seed)
        }
        _ => {
            println!("Unknown method '{}'. Use 'grid' or 'random'.", method);
            return Ok(());
        }
    };

    // Clear progress line
    println!("\n");

    // Print best configuration
    println!("{}", best);

    // Print top results
    print_top_results(&results, top);

    // Export to CSV if requested
    if let Some(path) = export {
        export_results_to_csv(&results, &path)?;
        println!("\nResults exported to: {}", path);
    }

    Ok(())
}

/// Runs a single backtest with specified parameters.
async fn run_backtest(
    database: &Database,
    k_factor: f64,
    finish: f64,
    title: f64,
    five_round: f64,
) -> anyhow::Result<()> {
    use crate::engine::Backtester;

    println!("Loading fight data...");
    let fights = database.get_fights_order_by_date().await?;

    if fights.is_empty() {
        println!("No fight data found. Run 'sync' first:");
        println!("  cargo run -- sync");
        return Ok(());
    }

    let config = EloConfig::new(k_factor, finish, title, five_round);
    println!("Running backtest with configuration:");
    println!("  K-Factor:              {:.2}", config.k_factor);
    println!("  Finish Multiplier:     {:.2}", config.finish_multiplier);
    println!(
        "  Title Fight Multiplier:{:.2}",
        config.title_fight_multiplier
    );
    println!(
        "  Five Round Multiplier: {:.2}",
        config.five_round_multiplier
    );
    println!();

    let mut backtester = Backtester::new();
    let result = backtester.run(&fights, &config);

    println!("Backtest Results:");
    println!("  Fights Processed: {}", result.fights_processed);
    println!("  Log Loss:         {:.4}", result.metrics.log_loss);
    println!("  Brier Score:      {:.4}", result.metrics.brier_score);
    println!(
        "  Accuracy:         {:.2}%",
        result.metrics.accuracy * 100.0
    );

    Ok(())
}
