use anyhow::Context;
use clap::Parser;
use sui_indexer_alt_framework::{
    ingestion::{
        ingestion_client::IngestionClientArgs, streaming_client::StreamingClientArgs, ClientArgs,
        IngestionConfig,
    },
    pipeline::concurrent::ConcurrentConfig,
    Indexer, IndexerArgs, TaskArgs,
};
use tracing::info;

use vortex_indexer::{
    handlers::{NewCommitmentHandler, NewPoolHandler, NullifierSpentHandler},
    parse_package_address,
    store::MongoStore,
    SuiNetwork, VortexEnv,
};

const DEFAULT_VORTEX_PACKAGE: &str =
    "0xd9d3b65c318e7d7dd208050a28e113a45256765b4c45acd119626d8a228d7555";
const DEFAULT_FIRST_CHECKPOINT: &str = "289686776";

#[derive(Parser)]
#[clap(
    name = "vortex-indexer",
    about = "Vortex Protocol Indexer for Sui using MongoDB"
)]
struct Config {
    #[clap(long, env, default_value = "mongodb://localhost:27017")]
    mongodb_uri: String,

    #[clap(long, env, default_value = "vortex")]
    mongodb_database: String,

    #[clap(long, env, default_value = "testnet")]
    sui_network: SuiNetwork,

    #[clap(long, env, default_value = DEFAULT_VORTEX_PACKAGE)]
    vortex_package: String,

    #[clap(long, env, default_value = DEFAULT_FIRST_CHECKPOINT)]
    first_checkpoint: u64,

    #[clap(long, env)]
    last_checkpoint: Option<u64>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let _ = dotenvy::dotenv();

    let _guard = telemetry_subscribers::TelemetryConfig::new()
        .with_env()
        .init();

    let config = Config::parse();

    let package_address =
        parse_package_address(&config.vortex_package).context("Invalid VORTEX_PACKAGE address")?;

    let env = VortexEnv::new(config.sui_network, package_address);

    info!(
        network = %config.sui_network,
        package = %config.vortex_package,
        mongodb = %config.mongodb_uri,
        database = %config.mongodb_database,
        "Starting Vortex Indexer"
    );

    let store = MongoStore::new(&config.mongodb_uri, &config.mongodb_database)
        .await
        .context("Failed to connect to MongoDB")?;

    info!("Connected to MongoDB");

    let client_args = ClientArgs {
        ingestion: IngestionClientArgs {
            remote_store_url: Some(env.remote_store_url()),
            local_ingestion_path: None,
            rpc_api_url: None,
            rpc_username: None,
            rpc_password: None,
        },
        streaming: StreamingClientArgs {
            streaming_url: Some(env.streaming_url().to_string().parse().expect("valid URI")),
        },
    };

    let indexer_args = IndexerArgs {
        first_checkpoint: Some(config.first_checkpoint),
        last_checkpoint: config.last_checkpoint,
        pipeline: vec![],
        task: TaskArgs::default(),
    };

    let mut indexer = Indexer::new(
        store,
        indexer_args,
        client_args,
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

    indexer
        .run()
        .await
        .context("Failed to start indexer")?
        .join()
        .await?;

    info!("Indexer stopped");

    Ok(())
}
