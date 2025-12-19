use mongodb::bson;
use serde::{Deserialize, Serialize};

pub mod collections {
    pub const NEW_POOLS: &str = "new_pools";
    pub const NEW_COMMITMENTS: &str = "new_commitments";
    pub const NULLIFIERS_SPENT: &str = "nullifiers_spent";
    pub const WATERMARKS: &str = "watermarks";
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventBase {
    #[serde(rename = "_id")]
    pub event_digest: String,
    pub digest: String,
    pub sender: String,
    pub checkpoint: u64,
    pub checkpoint_timestamp_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewPool {
    #[serde(flatten)]
    pub base: EventBase,
    pub pool_address: String,
    pub coin_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewCommitment {
    #[serde(flatten)]
    pub base: EventBase,
    pub coin_type: String,
    pub index: u64,
    pub commitment: String,
    pub encrypted_output: Vec<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NullifierSpent {
    #[serde(flatten)]
    pub base: EventBase,
    pub coin_type: String,
    pub nullifier: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Watermark {
    #[serde(rename = "_id")]
    pub pipeline: String,
    pub epoch_hi_inclusive: u64,
    pub checkpoint_hi_inclusive: u64,
    pub tx_hi: u64,
    pub timestamp_ms_hi_inclusive: u64,
    pub reader_lo: u64,
    pub pruner_hi: u64,
    pub pruner_timestamp: bson::DateTime,
}

impl Watermark {
    pub fn new(pipeline: String, default_checkpoint: u64) -> Self {
        Self {
            pipeline,
            epoch_hi_inclusive: 0,
            checkpoint_hi_inclusive: default_checkpoint.saturating_sub(1),
            tx_hi: 0,
            timestamp_ms_hi_inclusive: 0,
            reader_lo: default_checkpoint,
            pruner_hi: default_checkpoint,
            pruner_timestamp: bson::DateTime::now(),
        }
    }
}
