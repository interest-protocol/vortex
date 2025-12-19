use anyhow::{Context, Result};
use mongodb::{
    bson::doc,
    options::{ClientOptions, FindOneAndUpdateOptions, IndexOptions, InsertManyOptions},
    Client, Collection, Database, IndexModel,
};
use serde::{de::DeserializeOwned, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};
use vortex_schema::{collections, IndexerState};

const INDEXER_STATE_ID: &str = "vortex-indexer";

#[derive(Clone)]
pub struct MongoStore {
    database: Database,
}

impl MongoStore {
    pub async fn new(uri: &str, db_name: &str) -> Result<Self> {
        let mut client_options = ClientOptions::parse(uri)
            .await
            .context("Failed to parse MongoDB URI")?;

        client_options.app_name = Some("vortex-indexer".to_string());

        let client =
            Client::with_options(client_options).context("Failed to create MongoDB client")?;

        // Verify connection
        client
            .database("admin")
            .run_command(doc! { "ping": 1 })
            .await
            .context("Failed to ping MongoDB")?;

        let database = client.database(db_name);
        let store = Self { database };
        store.create_indexes().await?;

        Ok(store)
    }

    async fn create_indexes(&self) -> Result<()> {
        // NewCommitments: compound index for merkle tree queries by coin_type and leaf index
        self.create_index::<vortex_schema::NewCommitment>(
            collections::NEW_COMMITMENTS,
            doc! { "coin_type": 1, "index": 1 },
            Some("coin_type_index_idx"),
            false,
        )
        .await?;

        // NewCommitments: index for querying by checkpoint (useful for sync status)
        self.create_index::<vortex_schema::NewCommitment>(
            collections::NEW_COMMITMENTS,
            doc! { "checkpoint": 1 },
            Some("checkpoint_idx"),
            false,
        )
        .await?;

        // NullifiersSpent: unique index by coin_type and nullifier (prevents double-spend)
        self.create_index::<vortex_schema::NullifierSpent>(
            collections::NULLIFIERS_SPENT,
            doc! { "coin_type": 1, "nullifier": 1 },
            Some("coin_type_nullifier_idx"),
            true,
        )
        .await?;

        // NewPools: index by coin_type for pool lookups
        self.create_index::<vortex_schema::NewPool>(
            collections::NEW_POOLS,
            doc! { "coin_type": 1 },
            Some("coin_type_idx"),
            false,
        )
        .await?;

        // NewPools: unique index by pool_address (each pool is unique)
        self.create_index::<vortex_schema::NewPool>(
            collections::NEW_POOLS,
            doc! { "pool_address": 1 },
            Some("pool_address_idx"),
            true,
        )
        .await?;

        Ok(())
    }

    async fn create_index<T>(
        &self,
        collection_name: &str,
        keys: mongodb::bson::Document,
        name: Option<&str>,
        unique: bool,
    ) -> Result<()>
    where
        T: Serialize + DeserializeOwned + Send + Sync,
    {
        let collection: Collection<T> = self.database.collection(collection_name);

        let options = IndexOptions::builder()
            .name(name.map(String::from))
            .unique(unique)
            .build();

        let index = IndexModel::builder().keys(keys).options(options).build();

        collection
            .create_index(index)
            .await
            .context(format!("Failed to create index on {}", collection_name))?;

        Ok(())
    }

    /// Get the last processed checkpoint, returns None if no checkpoint has been processed
    pub async fn get_last_checkpoint(&self) -> Result<Option<u64>> {
        let collection: Collection<IndexerState> =
            self.database.collection(collections::INDEXER_STATE);

        let result = collection
            .find_one(doc! { "_id": INDEXER_STATE_ID })
            .await
            .context("Failed to query indexer state")?;

        Ok(result.map(|state| state.last_checkpoint))
    }

    /// Save the last processed checkpoint
    pub async fn save_checkpoint(&self, checkpoint: u64) -> Result<()> {
        let collection: Collection<IndexerState> =
            self.database.collection(collections::INDEXER_STATE);

        let now_ms = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;

        let options = FindOneAndUpdateOptions::builder().upsert(true).build();

        collection
            .find_one_and_update(
                doc! { "_id": INDEXER_STATE_ID },
                doc! {
                    "$set": {
                        "last_checkpoint": checkpoint as i64,
                        "last_updated_ms": now_ms as i64,
                    }
                },
            )
            .with_options(options)
            .await
            .context("Failed to save checkpoint")?;

        Ok(())
    }

    /// Insert many documents, handling duplicate key errors gracefully
    pub async fn insert_many<T>(&self, collection_name: &str, docs: Vec<T>) -> Result<usize>
    where
        T: Serialize + Send + Sync,
    {
        if docs.is_empty() {
            return Ok(0);
        }

        let collection: Collection<T> = self.database.collection(collection_name);
        let options = InsertManyOptions::builder().ordered(false).build();

        match collection.insert_many(&docs).with_options(options).await {
            Ok(result) => Ok(result.inserted_ids.len()),
            Err(e) => {
                // Handle partial success (some docs already exist)
                if let mongodb::error::ErrorKind::BulkWrite(ref bulk_err) = *e.kind {
                    let inserted = docs.len() - bulk_err.write_errors.len();
                    return Ok(inserted);
                }
                Err(e.into())
            }
        }
    }
}
