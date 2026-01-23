use anyhow::{Context, Result};
use async_trait::async_trait;
use mongodb::{
    bson::{self, doc},
    options::{ClientOptions, IndexOptions},
    Client, Collection, Database, IndexModel,
};
use scoped_futures::ScopedBoxFuture;
use serde::{de::DeserializeOwned, Serialize};
use std::time::Duration;
use sui_indexer_alt_framework_store_traits::{
    CommitterWatermark, Connection, PrunerWatermark, ReaderWatermark, Store, TransactionalStore,
};
use tracing::debug;
use vortex_schema::{collections, Watermark};

#[derive(Clone)]
pub struct MongoStore {
    database: Database,
}

pub struct MongoConnection {
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

        client
            .database("admin")
            .run_command(doc! { "ping": 1 })
            .await
            .context("Failed to ping MongoDB")?;

        let database = client.database(db_name);
        let store = Self { database };
        store.create_indexes().await?;

        debug!(db_name, "MongoDB store initialized");

        Ok(store)
    }

    #[must_use]
    pub fn database(&self) -> &Database {
        &self.database
    }

    async fn create_indexes(&self) -> Result<()> {
        self.create_index::<vortex_schema::NewCommitment>(
            collections::NEW_COMMITMENTS,
            doc! { "coin_type": 1, "index": 1 },
            Some("coin_type_index_idx"),
            false,
        )
        .await?;

        self.create_index::<vortex_schema::NewCommitment>(
            collections::NEW_COMMITMENTS,
            doc! { "checkpoint": 1 },
            Some("checkpoint_idx"),
            false,
        )
        .await?;

        self.create_index::<vortex_schema::NullifierSpent>(
            collections::NULLIFIERS_SPENT,
            doc! { "coin_type": 1, "nullifier": 1 },
            Some("coin_type_nullifier_idx"),
            true,
        )
        .await?;

        self.create_index::<vortex_schema::NewPool>(
            collections::NEW_POOLS,
            doc! { "coin_type": 1 },
            Some("coin_type_idx"),
            false,
        )
        .await?;

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
            .with_context(|| format!("Failed to create index on {collection_name}"))?;

        Ok(())
    }
}

#[async_trait]
impl Store for MongoStore {
    type Connection<'c> = MongoConnection;

    async fn connect<'c>(&'c self) -> Result<Self::Connection<'c>> {
        Ok(MongoConnection {
            database: self.database.clone(),
        })
    }
}

#[async_trait]
impl TransactionalStore for MongoStore {
    async fn transaction<'a, R, F>(&self, f: F) -> Result<R>
    where
        R: Send + 'a,
        F: Send + 'a,
        F: for<'r> FnOnce(&'r mut Self::Connection<'_>) -> ScopedBoxFuture<'a, 'r, Result<R>>,
    {
        let mut conn = self.connect().await?;
        f(&mut conn).await
    }
}

impl MongoConnection {
    #[must_use]
    pub fn database(&self) -> &Database {
        &self.database
    }

    fn watermarks(&self) -> Collection<Watermark> {
        self.database.collection(collections::WATERMARKS)
    }
}

#[async_trait]
impl Connection for MongoConnection {
    async fn committer_watermark(
        &mut self,
        pipeline_task: &str,
    ) -> Result<Option<CommitterWatermark>> {
        let result = self
            .watermarks()
            .find_one(doc! { "_id": pipeline_task })
            .await
            .context("Failed to query committer watermark")?;

        Ok(result.map(|w| CommitterWatermark {
            epoch_hi_inclusive: w.epoch_hi_inclusive,
            checkpoint_hi_inclusive: w.checkpoint_hi_inclusive,
            tx_hi: w.tx_hi,
            timestamp_ms_hi_inclusive: w.timestamp_ms_hi_inclusive,
        }))
    }

    async fn reader_watermark(
        &mut self,
        pipeline: &'static str,
    ) -> Result<Option<ReaderWatermark>> {
        let result = self
            .watermarks()
            .find_one(doc! { "_id": pipeline })
            .await
            .context("Failed to query reader watermark")?;

        Ok(result.map(|w| ReaderWatermark {
            checkpoint_hi_inclusive: w.checkpoint_hi_inclusive,
            reader_lo: w.reader_lo,
        }))
    }

    async fn pruner_watermark(
        &mut self,
        pipeline: &'static str,
        delay: Duration,
    ) -> Result<Option<PrunerWatermark>> {
        let result = self
            .watermarks()
            .find_one(doc! { "_id": pipeline })
            .await
            .context("Failed to query pruner watermark")?;

        let Some(w) = result else {
            return Ok(None);
        };

        let now = bson::DateTime::now();
        let elapsed_ms = now.timestamp_millis() - w.pruner_timestamp.timestamp_millis();
        let delay_ms = delay.as_millis() as i64;
        let wait_for_ms = delay_ms - elapsed_ms;

        Ok(Some(PrunerWatermark {
            wait_for_ms,
            pruner_hi: w.pruner_hi,
            reader_lo: w.reader_lo,
        }))
    }

    async fn set_committer_watermark(
        &mut self,
        pipeline_task: &str,
        watermark: CommitterWatermark,
    ) -> Result<bool> {
        let result = self
            .watermarks()
            .update_one(
                doc! {
                    "_id": pipeline_task,
                    "checkpoint_hi_inclusive": { "$lt": watermark.checkpoint_hi_inclusive as i64 }
                },
                doc! {
                    "$set": {
                        "epoch_hi_inclusive": watermark.epoch_hi_inclusive as i64,
                        "checkpoint_hi_inclusive": watermark.checkpoint_hi_inclusive as i64,
                        "tx_hi": watermark.tx_hi as i64,
                        "timestamp_ms_hi_inclusive": watermark.timestamp_ms_hi_inclusive as i64,
                    }
                },
            )
            .await
            .context("Failed to set committer watermark")?;

        Ok(result.modified_count > 0)
    }

    async fn set_reader_watermark(
        &mut self,
        pipeline: &'static str,
        reader_lo: u64,
    ) -> Result<bool> {
        let now = bson::DateTime::now();
        let result = self
            .watermarks()
            .update_one(
                doc! {
                    "_id": pipeline,
                    "reader_lo": { "$lt": reader_lo as i64 }
                },
                doc! {
                    "$set": {
                        "reader_lo": reader_lo as i64,
                        "pruner_timestamp": now,
                    }
                },
            )
            .await
            .context("Failed to set reader watermark")?;

        Ok(result.modified_count > 0)
    }

    async fn set_pruner_watermark(
        &mut self,
        pipeline: &'static str,
        pruner_hi: u64,
    ) -> Result<bool> {
        let result = self
            .watermarks()
            .update_one(
                doc! {
                    "_id": pipeline,
                    "pruner_hi": { "$lt": pruner_hi as i64 }
                },
                doc! { "$set": { "pruner_hi": pruner_hi as i64 } },
            )
            .await
            .context("Failed to set pruner watermark")?;

        Ok(result.modified_count > 0)
    }

    async fn init_watermark(&mut self, pipeline: &str, start: u64) -> Result<Option<u64>> {
        let now = bson::DateTime::now();

        self.watermarks()
            .update_one(
                doc! { "_id": pipeline },
                doc! {
                    "$setOnInsert": {
                        "_id": pipeline,
                        "epoch_hi_inclusive": 0_i64,
                        "checkpoint_hi_inclusive": (start as i64) - 1,
                        "tx_hi": 0_i64,
                        "timestamp_ms_hi_inclusive": 0_i64,
                        "reader_lo": 0_i64,
                        "pruner_hi": 0_i64,
                        "pruner_timestamp": now,
                    }
                },
            )
            .upsert(true)
            .await
            .context("Failed to init watermark")?;

        let result = self
            .watermarks()
            .find_one(doc! { "_id": pipeline })
            .await
            .context("Failed to query watermark after init")?;

        Ok(result.map(|w| w.checkpoint_hi_inclusive))
    }
}
