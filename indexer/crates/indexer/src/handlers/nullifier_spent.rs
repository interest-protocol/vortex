use crate::handlers::{process_vortex_events, u256_to_hex};
use crate::models::NullifierSpentEvent;
use crate::VortexEnv;
use anyhow::Result;
use async_trait::async_trait;
use std::sync::Arc;
use sui_indexer_alt_framework::pipeline::Processor;
use sui_types::full_checkpoint_content::Checkpoint;
use vortex_schema::{EventBase, NullifierSpent};

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
            |event: NullifierSpentEvent,
             digest,
             sender,
             coin_type,
             checkpoint_seq,
             checkpoint_ts,
             idx| {
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

crate::impl_mongo_handler!(
    NullifierSpentHandler,
    NullifierSpent,
    vortex_schema::collections::NULLIFIERS_SPENT
);
