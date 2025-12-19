use crate::models::{
    bytes_to_address, extract_coin_type, u256_to_hex, Coin, NewCommitmentEvent, NewPoolEvent,
    NullifierSpentEvent,
};
use crate::store::MongoStore;
use crate::traits::MoveStruct;
use anyhow::Result;
use move_core_types::account_address::AccountAddress;
use std::sync::Arc;
use sui_types::full_checkpoint_content::{CheckpointData, CheckpointTransaction};
use tracing::debug;
use vortex_schema::{collections, EventBase, NewCommitment, NewPool, NullifierSpent};

/// Check if transaction involves Vortex package
pub fn is_vortex_tx(tx: &CheckpointTransaction, package_address: &AccountAddress) -> bool {
    tx.events
        .as_ref()
        .map(|events| {
            events
                .data
                .iter()
                .any(|e| &e.type_.address == package_address)
        })
        .unwrap_or(false)
}

/// Process a single checkpoint and insert all events into MongoDB
pub async fn process_checkpoint(
    store: &Arc<MongoStore>,
    checkpoint: &Arc<CheckpointData>,
    package_address: &AccountAddress,
) -> Result<()> {
    let mut new_pools = Vec::new();
    let mut new_commitments = Vec::new();
    let mut nullifiers_spent = Vec::new();

    let checkpoint_seq = checkpoint.checkpoint_summary.sequence_number;
    let checkpoint_ts = checkpoint.checkpoint_summary.timestamp_ms;

    for tx in &checkpoint.transactions {
        if !is_vortex_tx(tx, package_address) {
            continue;
        }

        let Some(events) = &tx.events else {
            continue;
        };

        let digest = tx.transaction.digest().to_string();
        let sender = tx.transaction.sender_address().to_string();

        for (idx, ev) in events.data.iter().enumerate() {
            if &ev.type_.address != package_address {
                continue;
            }

            let event_digest = format!("{}:{}", digest, idx);
            let package = ev.type_.address.to_string();

            // NewPool
            if NewPoolEvent::<Coin>::matches_event_type(&ev.type_, package_address) {
                if let Ok(event) = bcs::from_bytes::<NewPoolEvent<Coin>>(&ev.contents) {
                    let coin_type = extract_coin_type(&ev.type_.to_string()).unwrap_or_default();
                    let pool_addr = bytes_to_address(&event.0);
                    new_pools.push(NewPool {
                        base: EventBase {
                            event_digest,
                            digest: digest.clone(),
                            sender: sender.clone(),
                            checkpoint: checkpoint_seq,
                            checkpoint_timestamp_ms: checkpoint_ts,
                            package: package.clone(),
                        },
                        pool_address: pool_addr.to_string(),
                        coin_type,
                    });
                    debug!("NewPool: {}", pool_addr);
                }
                continue;
            }

            // NewCommitment
            if NewCommitmentEvent::<Coin>::matches_event_type(&ev.type_, package_address) {
                if let Ok(event) = bcs::from_bytes::<NewCommitmentEvent<Coin>>(&ev.contents) {
                    let coin_type = extract_coin_type(&ev.type_.to_string()).unwrap_or_default();
                    new_commitments.push(NewCommitment {
                        base: EventBase {
                            event_digest,
                            digest: digest.clone(),
                            sender: sender.clone(),
                            checkpoint: checkpoint_seq,
                            checkpoint_timestamp_ms: checkpoint_ts,
                            package: package.clone(),
                        },
                        coin_type,
                        index: event.index,
                        commitment: u256_to_hex(&event.commitment),
                        encrypted_output: event.encrypted_output,
                    });
                    debug!("NewCommitment: index={}", event.index);
                }
                continue;
            }

            // NullifierSpent
            if NullifierSpentEvent::<Coin>::matches_event_type(&ev.type_, package_address) {
                if let Ok(event) = bcs::from_bytes::<NullifierSpentEvent<Coin>>(&ev.contents) {
                    let coin_type = extract_coin_type(&ev.type_.to_string()).unwrap_or_default();
                    nullifiers_spent.push(NullifierSpent {
                        base: EventBase {
                            event_digest,
                            digest: digest.clone(),
                            sender: sender.clone(),
                            checkpoint: checkpoint_seq,
                            checkpoint_timestamp_ms: checkpoint_ts,
                            package: package.clone(),
                        },
                        coin_type,
                        nullifier: u256_to_hex(&event.0),
                    });
                    debug!("NullifierSpent: {}", u256_to_hex(&event.0));
                }
                continue;
            }
        }
    }

    let (pools_res, commits_res, nulls_res) = tokio::try_join!(
        async {
            if new_pools.is_empty() {
                Ok(0)
            } else {
                store.insert_many(collections::NEW_POOLS, new_pools).await
            }
        },
        async {
            if new_commitments.is_empty() {
                Ok(0)
            } else {
                store
                    .insert_many(collections::NEW_COMMITMENTS, new_commitments)
                    .await
            }
        },
        async {
            if nullifiers_spent.is_empty() {
                Ok(0)
            } else {
                store
                    .insert_many(collections::NULLIFIERS_SPENT, nullifiers_spent)
                    .await
            }
        }
    )?;

    if pools_res > 0 || commits_res > 0 || nulls_res > 0 {
        debug!(
            "Checkpoint {}: {} pools, {} commitments, {} nullifiers",
            checkpoint_seq, pools_res, commits_res, nulls_res
        );
    }

    Ok(())
}
