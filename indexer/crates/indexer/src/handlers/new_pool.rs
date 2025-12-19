use crate::handlers::{bytes_to_address, extract_coin_type, is_vortex_tx};
use crate::models::NewPoolEvent;
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
                    || ev.type_.name.as_str() != "NewPool"
                {
                    continue;
                }

                let event = match bcs::from_bytes::<NewPoolEvent>(&ev.contents) {
                    Ok(e) => e,
                    Err(e) => {
                        warn!(
                            checkpoint = checkpoint_seq,
                            digest = %digest,
                            event_idx = idx,
                            error = %e,
                            "Failed to deserialize NewPool event"
                        );
                        continue;
                    }
                };

                let coin_type = extract_coin_type(&ev.type_.to_string()).unwrap_or_default();
                let pool_addr = bytes_to_address(&event.0);

                results.push(NewPool {
                    base: EventBase {
                        event_digest: format!("{digest}:{idx}"),
                        digest: digest.clone(),
                        sender: sender.clone(),
                        checkpoint: checkpoint_seq,
                        checkpoint_timestamp_ms: checkpoint_ts,
                    },
                    pool_address: pool_addr.to_string(),
                    coin_type,
                });
            }
        }

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
        if batch.is_empty() {
            return Ok(0);
        }

        let collection = conn
            .database()
            .collection::<NewPool>(collections::NEW_POOLS);

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
