use anyhow::Context;
use clap::Parser;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::watch;
use tracing::{debug, error, info, warn};

use vortex_indexer::{
    handlers::process_checkpoint, parse_package_address, store::MongoStore, SuiNetwork,
};

/// Default Vortex package address (testnet)
const DEFAULT_VORTEX_PACKAGE: &str =
    "0xcf81b96e392f82b776ee980108357426b726c4043c838822545a307e12c5ded6";

#[derive(Parser)]
#[clap(name = "vortex-indexer", about = "Vortex Protocol Indexer for Sui")]
struct Args {
    /// MongoDB connection URI
    #[clap(long, env = "MONGODB_URI", default_value = "mongodb://localhost:27017")]
    mongodb_uri: String,

    /// MongoDB database name
    #[clap(long, env = "MONGODB_DATABASE", default_value = "vortex")]
    mongodb_database: String,

    /// Sui network to index (mainnet, testnet, devnet)
    #[clap(long, env = "SUI_NETWORK", default_value = "testnet")]
    sui_network: SuiNetwork,

    /// Vortex package address to index
    #[clap(long, env = "VORTEX_PACKAGE", default_value = DEFAULT_VORTEX_PACKAGE)]
    vortex_package: String,

    /// Starting checkpoint number (overrides saved checkpoint)
    #[clap(long, env = "START_CHECKPOINT")]
    start_checkpoint: Option<u64>,

    /// Save checkpoint progress every N checkpoints
    #[clap(long, env = "CHECKPOINT_SAVE_INTERVAL", default_value = "100")]
    checkpoint_save_interval: u64,

    /// Log progress every N checkpoints
    #[clap(long, env = "LOG_INTERVAL", default_value = "1000")]
    log_interval: u64,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Load .env file if present (silently ignore if not found)
    let _ = dotenvy::dotenv();

    // Initialize telemetry/logging
    let _guard = telemetry_subscribers::TelemetryConfig::new()
        .with_env()
        .init();

    let args = Args::parse();

    // Parse package address
    let package_address =
        parse_package_address(&args.vortex_package).context("Invalid VORTEX_PACKAGE address")?;

    info!("Starting Vortex Indexer");
    info!("Network: {}", args.sui_network);
    info!("Checkpoint URL: {}", args.sui_network.checkpoint_url());
    info!("Package: {}", args.vortex_package);
    info!("MongoDB: {}/{}", args.mongodb_uri, args.mongodb_database);

    // Connect to MongoDB
    let store = MongoStore::new(&args.mongodb_uri, &args.mongodb_database)
        .await
        .context("Failed to connect to MongoDB")?;

    info!("Connected to MongoDB");

    // Determine starting checkpoint
    let start_checkpoint = match args.start_checkpoint {
        Some(cp) => {
            info!("Starting from checkpoint {} (CLI override)", cp);
            cp
        }
        None => {
            // Try to resume from last saved checkpoint
            match store.get_last_checkpoint().await? {
                Some(last_cp) => {
                    let resume_from = last_cp + 1;
                    info!(
                        "Resuming from checkpoint {} (last saved: {})",
                        resume_from, last_cp
                    );
                    resume_from
                }
                None => {
                    info!("No saved checkpoint found, starting from 0");
                    0
                }
            }
        }
    };

    // Setup graceful shutdown
    let (shutdown_tx, mut shutdown_rx) = watch::channel(false);

    tokio::spawn(async move {
        let ctrl_c = async {
            tokio::signal::ctrl_c()
                .await
                .expect("Failed to install Ctrl+C handler");
        };

        #[cfg(unix)]
        let terminate = async {
            tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
                .expect("Failed to install SIGTERM handler")
                .recv()
                .await;
        };

        #[cfg(not(unix))]
        let terminate = std::future::pending::<()>();

        tokio::select! {
            _ = ctrl_c => info!("Received Ctrl+C, shutting down..."),
            _ = terminate => info!("Received SIGTERM, shutting down..."),
        }

        let _ = shutdown_tx.send(true);
    });

    // Run the indexer
    let store = Arc::new(store);
    let result = run_indexer(
        store,
        &args,
        start_checkpoint,
        &package_address,
        &mut shutdown_rx,
    )
    .await;

    match &result {
        Ok(final_checkpoint) => {
            info!("Indexer stopped at checkpoint {}", final_checkpoint);
        }
        Err(e) => {
            error!("Indexer stopped with error: {:?}", e);
        }
    }

    result.map(|_| ())
}

async fn run_indexer(
    store: Arc<MongoStore>,
    args: &Args,
    start_checkpoint: u64,
    package_address: &move_core_types::account_address::AccountAddress,
    shutdown_rx: &mut watch::Receiver<bool>,
) -> anyhow::Result<u64> {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(30))
        .build()?;

    let base_url = args.sui_network.checkpoint_url();
    let mut checkpoint = start_checkpoint;
    let mut last_saved_checkpoint = start_checkpoint.saturating_sub(1);
    let mut consecutive_errors = 0u32;

    loop {
        // Check for shutdown signal
        if *shutdown_rx.borrow() {
            // Save final checkpoint before exiting
            if checkpoint > 0 && checkpoint - 1 > last_saved_checkpoint {
                store.save_checkpoint(checkpoint - 1).await?;
            }
            return Ok(checkpoint.saturating_sub(1));
        }

        let url = format!("{}/{:020}.chk", base_url, checkpoint);

        match client.get(&url).send().await {
            Ok(response) if response.status().is_success() => {
                consecutive_errors = 0;

                let bytes = response
                    .bytes()
                    .await
                    .context("Failed to read checkpoint bytes")?;

                match bcs::from_bytes::<sui_types::full_checkpoint_content::CheckpointData>(&bytes)
                {
                    Ok(data) => {
                        let data = Arc::new(data);

                        if let Err(e) = process_checkpoint(&store, &data, package_address).await {
                            error!("Failed to process checkpoint {}: {:?}", checkpoint, e);
                            // Continue to next checkpoint even if processing fails
                            // The events will be missing but we can reindex later if needed
                        }

                        // Progress logging
                        if checkpoint % args.log_interval == 0 {
                            info!("Processed checkpoint {}", checkpoint);
                        }

                        // Periodic checkpoint save
                        if checkpoint > 0
                            && checkpoint % args.checkpoint_save_interval == 0
                            && checkpoint > last_saved_checkpoint
                        {
                            store.save_checkpoint(checkpoint).await?;
                            last_saved_checkpoint = checkpoint;
                            debug!("Saved checkpoint progress: {}", checkpoint);
                        }

                        checkpoint += 1;
                    }
                    Err(e) => {
                        error!("Failed to deserialize checkpoint {}: {:?}", checkpoint, e);
                        // This is likely a corrupted checkpoint, skip it
                        checkpoint += 1;
                    }
                }
            }
            Ok(response) if response.status() == reqwest::StatusCode::NOT_FOUND => {
                // Checkpoint not available yet - we're caught up
                debug!("Checkpoint {} not available, waiting...", checkpoint);

                // Save progress while waiting
                if checkpoint > 0 && checkpoint - 1 > last_saved_checkpoint {
                    store.save_checkpoint(checkpoint - 1).await?;
                    last_saved_checkpoint = checkpoint - 1;
                }

                // Wait with shutdown check
                tokio::select! {
                    _ = tokio::time::sleep(Duration::from_millis(500)) => {}
                    _ = shutdown_rx.changed() => {}
                }
            }
            Ok(response) => {
                consecutive_errors += 1;
                warn!(
                    "Unexpected status {} for checkpoint {} (error {} of 10)",
                    response.status(),
                    checkpoint,
                    consecutive_errors
                );

                if consecutive_errors >= 10 {
                    anyhow::bail!(
                        "Too many consecutive errors fetching checkpoint {}",
                        checkpoint
                    );
                }

                tokio::time::sleep(Duration::from_secs(1)).await;
            }
            Err(e) => {
                consecutive_errors += 1;
                warn!(
                    "Failed to fetch checkpoint {}: {:?} (error {} of 10)",
                    checkpoint, e, consecutive_errors
                );

                if consecutive_errors >= 10 {
                    anyhow::bail!(
                        "Too many consecutive errors fetching checkpoint {}: {:?}",
                        checkpoint,
                        e
                    );
                }

                // Exponential backoff
                let delay = Duration::from_secs(2u64.pow(consecutive_errors.min(5)));
                tokio::time::sleep(delay).await;
            }
        }
    }
}
