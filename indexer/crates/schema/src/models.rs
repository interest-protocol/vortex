use serde::{Deserialize, Serialize};

/// Base fields common to all events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventBase {
    /// Unique event identifier (tx_digest + event_index)
    #[serde(rename = "_id")]
    pub event_digest: String,
    /// Transaction digest
    pub digest: String,
    /// Transaction sender address
    pub sender: String,
    /// Checkpoint sequence number
    pub checkpoint: u64,
    /// Checkpoint timestamp in milliseconds
    pub checkpoint_timestamp_ms: u64,
}

/// NewPool event - emitted when a new privacy pool is created
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewPool {
    #[serde(flatten)]
    pub base: EventBase,
    pub pool_address: String,
    pub coin_type: String,
}

/// NewCommitment event - emitted when a new UTXO commitment is added to the merkle tree
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewCommitment {
    #[serde(flatten)]
    pub base: EventBase,
    pub coin_type: String,
    /// Merkle tree leaf index
    pub index: u64,
    /// Commitment hash (hex encoded)
    pub commitment: String,
    /// Encrypted output data for the recipient
    pub encrypted_output: Vec<u8>,
}

/// NullifierSpent event - emitted when a UTXO is spent
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NullifierSpent {
    #[serde(flatten)]
    pub base: EventBase,
    pub coin_type: String,
    /// Nullifier hash (hex encoded)
    pub nullifier: String,
}

/// Indexer state - tracks the last processed checkpoint
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexerState {
    #[serde(rename = "_id")]
    pub id: String,
    pub last_checkpoint: u64,
    pub last_updated_ms: u64,
}

pub mod collections {
    pub const NEW_POOLS: &str = "new_pools";
    pub const NEW_COMMITMENTS: &str = "new_commitments";
    pub const NULLIFIERS_SPENT: &str = "nullifiers_spent";
    pub const INDEXER_STATE: &str = "indexer_state";
}
