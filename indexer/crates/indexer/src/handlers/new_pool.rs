use crate::handlers::{bulk_insert_unordered, bytes_to_address, process_vortex_events};
use crate::models::NewPoolEvent;
use crate::store::MongoStore;
use crate::VortexEnv;
use anyhow::Result;
use async_trait::async_trait;
use std::sync::Arc;
use std::vec::IntoIter;
use sui_indexer_alt_framework::pipeline::concurrent::{BatchStatus, Handler};
use sui_indexer_alt_framework::pipeline::Processor;
use sui_indexer_alt_framework_store_traits::Store;
use sui_types::full_checkpoint_content::Checkpoint;
use vortex_schema::{collections, EventBase, NewPool};

pub struct NewPoolHandler {
    env: VortexEnv,
}

impl NewPoolHandler {
    #[must_use]
    pub const fn new(env: VortexEnv) -> Self {
        Self { env }
    }
}

#[async_trait]
impl Processor for NewPoolHandler {
    const NAME: &'static str = "new_pools";
    type Value = NewPool;

    async fn process(&self, checkpoint: &Arc<Checkpoint>) -> Result<Vec<Self::Value>> {
        let results = process_vortex_events(
            &checkpoint.transactions,
            &self.env.package_address,
            "NewPool",
            checkpoint.summary.sequence_number,
            checkpoint.summary.timestamp_ms,
            |event: NewPoolEvent, digest, sender, coin_type, checkpoint_seq, checkpoint_ts, idx| {
                let pool_addr = bytes_to_address(&event.0);
                NewPool {
                    base: EventBase {
                        event_digest: format!("{digest}:{idx}"),
                        digest,
                        sender,
                        checkpoint: checkpoint_seq,
                        checkpoint_timestamp_ms: checkpoint_ts,
                    },
                    pool_address: pool_addr.to_string(),
                    coin_type,
                }
            },
        );

        Ok(results)
    }
}

#[async_trait]
impl Handler for NewPoolHandler {
    type Store = MongoStore;
    type Batch = Vec<Self::Value>;

    fn batch(&self, batch: &mut Self::Batch, values: &mut IntoIter<Self::Value>) -> BatchStatus {
        batch.extend(values);
        BatchStatus::Pending
    }

    async fn commit<'a>(
        &self,
        batch: &Self::Batch,
        conn: &mut <Self::Store as Store>::Connection<'a>,
    ) -> Result<usize> {
        let collection = conn
            .database()
            .collection::<NewPool>(collections::NEW_POOLS);

        bulk_insert_unordered(&collection, batch).await
    }
}
