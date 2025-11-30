module vortex::vortex_events;

use std::ascii::String;
use sui::event::emit;

// === Events ===

public struct NewPool<phantom CoinType>(address) has copy, drop;

public struct NewAccount(address, u256) has copy, drop;

public struct NewCommitment<phantom CoinType> has copy, drop {
    index: u64,
    commitment: u256,
    encrypted_output: vector<u8>,
}

public struct NullifierSpent<phantom CoinType>(u256) has copy, drop;

public struct NewEncryptionKey(address, String) has copy, drop;

// === Package Functions ===

public(package) fun new_pool<CoinType>(address: address) {
    emit(NewPool<CoinType>(address));
}

public(package) fun new_account(address: address, hashed_secret: u256) {
    emit(NewAccount(address, hashed_secret));
}

public(package) fun new_commitment<CoinType>(
    index: u64,
    commitment: u256,
    encrypted_output: vector<u8>,
) {
    emit(NewCommitment<CoinType> { index, commitment, encrypted_output });
}

public(package) fun nullifier_spent<CoinType>(nullifier: u256) {
    emit(NullifierSpent<CoinType>(nullifier));
}

public(package) fun new_encryption_key(address: address, encryption_key: String) {
    emit(NewEncryptionKey(address, encryption_key));
}
