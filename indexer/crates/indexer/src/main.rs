use anyhow::Context;
use clap::Parser;
use sui_indexer_alt_framework::{
    ingestion::{ClientArgs, IngestionConfig},
    pipeline::concurrent::ConcurrentConfig,
    Indexer, IndexerArgs,
};
use tracing::info;

use vortex_indexer::{
    handlers::{NewCommitmentHandler, NewPoolHandler, NullifierSpentHandler},
    parse_package_address,
    store::MongoStore,
    SuiNetwork, VortexEnv,
};

const DEFAULT_VORTEX_PACKAGE: &str =
    "0xcf81b96e392f82b776ee980108357426b726c4043c838822545a307e12c5ded6";

#[derive(Parser)]
#[clap(
    name = "vortex-indexer",
    about = "Vortex Protocol Indexer for Sui using MongoDB"
)]
struct Args {
    #[clap(long, env = "MONGODB_URI", default_value = "mongodb://localhost:27017")]
    mongodb_uri: String,

    #[clap(long, env = "MONGODB_DATABASE", default_value = "vortex")]
    mongodb_database: String,

    #[clap(long, env = "SUI_NETWORK", default_value = "testnet")]
    sui_network: SuiNetwork,

    #[clap(long, env = "VORTEX_PACKAGE", default_value = DEFAULT_VORTEX_PACKAGE)]
    vortex_package: String,

    #[clap(flatten)]
    indexer_args: IndexerArgs,

    #[clap(flatten)]
    client_args: ClientArgs,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let _ = dotenvy::dotenv();

    let _guard = telemetry_subscribers::TelemetryConfig::new()
        .with_env()
        .init();

    let args = Args::parse();

    let package_address =
        parse_package_address(&args.vortex_package).context("Invalid VORTEX_PACKAGE address")?;

    let env = VortexEnv::new(args.sui_network, package_address);

    info!(
        network = %args.sui_network,
        package = %args.vortex_package,
        mongodb = %args.mongodb_uri,
        database = %args.mongodb_database,
        "Starting Vortex Indexer"
    );

    let store = MongoStore::new(&args.mongodb_uri, &args.mongodb_database)
        .await
        .context("Failed to connect to MongoDB")?;

    info!("Connected to MongoDB");

    let mut indexer = Indexer::new(
        store,
        args.indexer_args,
        args.client_args,
        IngestionConfig::default(),
        None,
        &prometheus::Registry::new(),
    )
    .await
    .context("Failed to create indexer")?;

    indexer
        .concurrent_pipeline(NewPoolHandler::new(env), ConcurrentConfig::default())
        .await
        .context("Failed to register NewPoolHandler pipeline")?;

    indexer
        .concurrent_pipeline(NewCommitmentHandler::new(env), ConcurrentConfig::default())
        .await
        .context("Failed to register NewCommitmentHandler pipeline")?;

    indexer
        .concurrent_pipeline(NullifierSpentHandler::new(env), ConcurrentConfig::default())
        .await
        .context("Failed to register NullifierSpentHandler pipeline")?;

    info!("All pipelines registered, starting indexer...");

    let mut service = indexer.run().await.context("Failed to start indexer")?;

    service.join().await.context("Indexer service failed")?;

    info!("Indexer stopped");

    Ok(())
}
