mod new_commitment;
mod new_pool;
mod nullifier_spent;

pub use new_commitment::NewCommitmentHandler;
pub use new_pool::NewPoolHandler;
pub use nullifier_spent::NullifierSpentHandler;

use anyhow::Result;
use mongodb::options::InsertManyOptions;
use once_cell::sync::Lazy;
use regex::Regex;
use serde::de::DeserializeOwned;
use sui_types::base_types::SuiAddress;
use sui_types::full_checkpoint_content::ExecutedTransaction;
use sui_types::transaction::TransactionDataAPI;
use tracing::warn;

#[macro_export]
macro_rules! impl_mongo_handler {
    ($handler:ty, $value:ty, $collection:expr) => {
        #[async_trait::async_trait]
        impl sui_indexer_alt_framework::pipeline::concurrent::Handler for $handler {
            type Store = $crate::store::MongoStore;
            type Batch = Vec<$value>;

            fn batch(
                &self,
                batch: &mut Self::Batch,
                values: &mut std::vec::IntoIter<$value>,
            ) -> sui_indexer_alt_framework::pipeline::concurrent::BatchStatus {
                batch.extend(values);
                sui_indexer_alt_framework::pipeline::concurrent::BatchStatus::Pending
            }

            async fn commit<'a>(
                &self,
                batch: &Self::Batch,
                conn: &mut <Self::Store as sui_indexer_alt_framework_store_traits::Store>::Connection<'a>,
            ) -> anyhow::Result<usize> {
                let collection = conn.database().collection::<$value>($collection);
                $crate::handlers::bulk_insert_unordered(&collection, batch).await
            }
        }
    };
}

pub fn is_vortex_tx(tx: &ExecutedTransaction, package_address: SuiAddress) -> bool {
    tx.events
        .as_ref()
        .map(|events| {
            events
                .data
                .iter()
                .any(|e| e.type_.address == package_address.into())
        })
        .unwrap_or(false)
}

pub fn u256_to_hex(value: &[u8; 32]) -> String {
    format!("0x{}", hex::encode(value))
}

pub fn bytes_to_address(bytes: &[u8; 32]) -> SuiAddress {
    SuiAddress::from_bytes(bytes).expect("32 bytes is valid SuiAddress")
}

static COIN_TYPE_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"<(.+)>").unwrap());

pub fn extract_coin_type(type_str: &str) -> Option<String> {
    COIN_TYPE_RE
        .captures(type_str)
        .and_then(|caps| caps.get(1))
        .map(|m| m.as_str().to_string())
}

/// Performs an unordered bulk insert into MongoDB, continuing on duplicate key errors.
/// Returns the number of successfully inserted documents.
pub async fn bulk_insert_unordered<T>(
    collection: &mongodb::Collection<T>,
    batch: &[T],
) -> Result<usize>
where
    T: serde::Serialize + Send + Sync,
{
    if batch.is_empty() {
        return Ok(0);
    }

    let options = InsertManyOptions::builder().ordered(false).build();

    match collection.insert_many(batch).with_options(options).await {
        Ok(result) => Ok(result.inserted_ids.len()),
        Err(e) => {
            if let mongodb::error::ErrorKind::InsertMany(ref insert_err) = *e.kind {
                let errors_count = insert_err
                    .write_errors
                    .as_ref()
                    .map(|errs| errs.len())
                    .unwrap_or(0);
                let inserted = batch.len().saturating_sub(errors_count);
                return Ok(inserted);
            }
            Err(e.into())
        }
    }
}

pub fn process_vortex_events<TEvent, TResult, F>(
    transactions: &[ExecutedTransaction],
    package_address: SuiAddress,
    event_name: &str,
    checkpoint_seq: u64,
    checkpoint_ts: u64,
    mut map_event: F,
) -> Vec<TResult>
where
    TEvent: DeserializeOwned,
    F: FnMut(TEvent, String, String, String, u64, u64, usize) -> TResult,
{
    let mut results = Vec::new();
    let account_address = package_address.into();

    for tx in transactions {
        if !is_vortex_tx(tx, package_address) {
            continue;
        }

        let Some(events) = &tx.events else {
            continue;
        };

        let digest = tx.transaction.digest().to_string();
        let sender = tx.transaction.sender().to_string();

        for (idx, ev) in events.data.iter().enumerate() {
            if ev.type_.address != account_address {
                continue;
            }

            if ev.type_.module.as_str() != "vortex_events" || ev.type_.name.as_str() != event_name {
                continue;
            }

            let event = match bcs::from_bytes::<TEvent>(&ev.contents) {
                Ok(e) => e,
                Err(e) => {
                    warn!(
                        checkpoint = checkpoint_seq,
                        digest = %digest,
                        event_idx = idx,
                        error = %e,
                        "Failed to deserialize {} event",
                        event_name
                    );
                    continue;
                }
            };

            let coin_type = extract_coin_type(&ev.type_.to_string()).unwrap_or_default();

            results.push(map_event(
                event,
                digest.clone(),
                sender.clone(),
                coin_type,
                checkpoint_seq,
                checkpoint_ts,
                idx,
            ));
        }
    }

    results
}
