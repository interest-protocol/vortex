use crate::handlers::{bulk_insert_unordered, process_vortex_events, u256_to_hex};
use crate::models::NullifierSpentEvent;
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
use vortex_schema::{collections, EventBase, NullifierSpent};

pub struct NullifierSpentHandler {
    env: VortexEnv,
}

impl NullifierSpentHandler {
    #[must_use]
    pub const fn new(env: VortexEnv) -> Self {
        Self { env }
    }
}

#[async_trait]
impl Processor for NullifierSpentHandler {
    const NAME: &'static str = "nullifiers_spent";
    type Value = NullifierSpent;

    async fn process(&self, checkpoint: &Arc<Checkpoint>) -> Result<Vec<Self::Value>> {
        let results = process_vortex_events(
            &checkpoint.transactions,
            &self.env.package_address,
            "NullifierSpent",
            checkpoint.summary.sequence_number,
            checkpoint.summary.timestamp_ms,
            |event: NullifierSpentEvent, digest, sender, coin_type, checkpoint_seq, checkpoint_ts, idx| {
                NullifierSpent {
                    base: EventBase {
                        event_digest: format!("{digest}:{idx}"),
                        digest,
                        sender,
                        checkpoint: checkpoint_seq,
                        checkpoint_timestamp_ms: checkpoint_ts,
                    },
                    coin_type,
                    nullifier: u256_to_hex(&event.0),
                }
            },
        );

        Ok(results)
    }
}

#[async_trait]
impl Handler for NullifierSpentHandler {
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
            .collection::<NullifierSpent>(collections::NULLIFIERS_SPENT);

        bulk_insert_unordered(&collection, batch).await
    }
}
