mod new_commitment;
mod new_pool;
mod nullifier_spent;

pub use new_commitment::NewCommitmentHandler;
pub use new_pool::NewPoolHandler;
pub use nullifier_spent::NullifierSpentHandler;

use move_core_types::account_address::AccountAddress;
use once_cell::sync::Lazy;
use regex::Regex;
use sui_types::full_checkpoint_content::ExecutedTransaction;

pub fn is_vortex_tx(tx: &ExecutedTransaction, package_address: &AccountAddress) -> bool {
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

pub fn u256_to_hex(value: &[u8; 32]) -> String {
    format!("0x{}", hex::encode(value))
}

pub fn bytes_to_address(bytes: &[u8; 32]) -> AccountAddress {
    AccountAddress::new(*bytes)
}

static COIN_TYPE_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"<(.+)>").unwrap());

pub fn extract_coin_type(type_str: &str) -> Option<String> {
    COIN_TYPE_RE
        .captures(type_str)
        .and_then(|caps| caps.get(1))
        .map(|m| m.as_str().to_string())
}
