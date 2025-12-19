use crate::handlers::{extract_coin_type, is_vortex_tx, u256_to_hex};
use crate::models::NewCommitmentEvent;
use crate::store::MongoStore;
use crate::VortexEnv;
use anyhow::Result;
use async_trait::async_trait;
use mongodb::options::InsertManyOptions;
use std::sync::Arc;
use std::vec::IntoIter;
use sui_indexer_alt_framework::pipeline::concurrent::{BatchStatus, Handler};
use sui_indexer_alt_framework::pipeline::Processor;
use sui_indexer_alt_framework_store_traits::Store;
use sui_types::full_checkpoint_content::Checkpoint;
use sui_types::transaction::TransactionDataAPI;
use tracing::warn;
use vortex_schema::{collections, EventBase, NewCommitment};

pub struct NewCommitmentHandler {
    env: VortexEnv,
}

impl NewCommitmentHandler {
    #[must_use]
    pub const fn new(env: VortexEnv) -> Self {
        Self { env }
    }
}

#[async_trait]
impl Processor for NewCommitmentHandler {
    const NAME: &'static str = "new_commitments";
    type Value = NewCommitment;

    async fn process(&self, checkpoint: &Arc<Checkpoint>) -> Result<Vec<Self::Value>> {
        let mut results = Vec::new();
        let checkpoint_seq = checkpoint.summary.sequence_number;
        let checkpoint_ts = checkpoint.summary.timestamp_ms;

        for tx in &checkpoint.transactions {
            if !is_vortex_tx(tx, &self.env.package_address) {
                continue;
            }

            let Some(events) = &tx.events else {
                continue;
            };

            let digest = tx.transaction.digest().to_string();
            let sender = tx.transaction.sender().to_string();

            for (idx, ev) in events.data.iter().enumerate() {
                if ev.type_.address != self.env.package_address {
                    continue;
                }

                if ev.type_.module.as_str() != "vortex_events"
                    || ev.type_.name.as_str() != "NewCommitment"
                {
                    continue;
                }

                let event = match bcs::from_bytes::<NewCommitmentEvent>(&ev.contents) {
                    Ok(e) => e,
                    Err(e) => {
                        warn!(
                            checkpoint = checkpoint_seq,
                            digest = %digest,
                            event_idx = idx,
                            error = %e,
                            "Failed to deserialize NewCommitment event"
                        );
                        continue;
                    }
                };

                let coin_type = extract_coin_type(&ev.type_.to_string()).unwrap_or_default();

                results.push(NewCommitment {
                    base: EventBase {
                        event_digest: format!("{digest}:{idx}"),
                        digest: digest.clone(),
                        sender: sender.clone(),
                        checkpoint: checkpoint_seq,
                        checkpoint_timestamp_ms: checkpoint_ts,
                    },
                    coin_type,
                    index: event.index,
                    commitment: u256_to_hex(&event.commitment),
                    encrypted_output: event.encrypted_output,
                });
            }
        }

        Ok(results)
    }
}

#[async_trait]
impl Handler for NewCommitmentHandler {
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
        if batch.is_empty() {
            return Ok(0);
        }

        let collection = conn
            .database()
            .collection::<NewCommitment>(collections::NEW_COMMITMENTS);

        let options = InsertManyOptions::builder().ordered(false).build();

        match collection.insert_many(batch).with_options(options).await {
            Ok(result) => Ok(result.inserted_ids.len()),
            Err(e) => {
                if let mongodb::error::ErrorKind::BulkWrite(ref bulk_err) = *e.kind {
                    let inserted = batch.len() - bulk_err.write_errors.len();
                    return Ok(inserted);
                }
                Err(e.into())
            }
        }
    }
}
