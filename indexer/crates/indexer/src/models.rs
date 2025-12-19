use once_cell::sync::Lazy;
use regex::Regex;
use serde::{Deserialize, Serialize};
use sui_sdk_types::Address;

use crate::traits::MoveStruct;

/// A 32-byte address for BCS deserialization
pub type AddressBytes = [u8; 32];

/// Helper to convert address bytes to Address type
pub fn bytes_to_address(bytes: &AddressBytes) -> Address {
    Address::new(*bytes)
}

/// NewPool<CoinType>(address) - tuple struct for BCS deserialization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewPoolEvent<CoinType>(
    pub AddressBytes,
    #[serde(skip)] pub std::marker::PhantomData<CoinType>,
);

impl<CoinType> MoveStruct for NewPoolEvent<CoinType> {
    const MODULE: &'static str = "vortex_events";
    const NAME: &'static str = "NewPool";
}

/// NewCommitment<CoinType> { index: u64, commitment: u256, encrypted_output: vector<u8> }
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewCommitmentEvent<CoinType> {
    pub index: u64,
    pub commitment: [u8; 32],
    pub encrypted_output: Vec<u8>,
    #[serde(skip)]
    pub phantom: std::marker::PhantomData<CoinType>,
}

impl<CoinType> MoveStruct for NewCommitmentEvent<CoinType> {
    const MODULE: &'static str = "vortex_events";
    const NAME: &'static str = "NewCommitment";
}

/// NullifierSpent<CoinType>(u256) - tuple struct for BCS deserialization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NullifierSpentEvent<CoinType>(
    pub [u8; 32],
    #[serde(skip)] pub std::marker::PhantomData<CoinType>,
);

impl<CoinType> MoveStruct for NullifierSpentEvent<CoinType> {
    const MODULE: &'static str = "vortex_events";
    const NAME: &'static str = "NullifierSpent";
}

/// Marker type for generic coin type parameter
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Coin;

pub fn u256_to_hex(bytes: &[u8; 32]) -> String {
    format!("0x{}", hex::encode(bytes))
}

static COIN_TYPE_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"<(.+)>").unwrap());

pub fn extract_coin_type(type_str: &str) -> Option<String> {
    COIN_TYPE_RE
        .captures(type_str)
        .and_then(|caps| caps.get(1))
        .map(|m| m.as_str().to_string())
}
