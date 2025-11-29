module vortex::vortex_account;

use sui::{coin::{Self, Coin}, transfer::Receiving};

// === Structs ===

public struct VortexAccount has key, store {
    id: UID,
    secret_hash: u256,
}

// === Public Mutative Functions ===

public fun new(secret_hash: u256, ctx: &mut TxContext): VortexAccount {
    VortexAccount {
        id: object::new(ctx),
        secret_hash,
    }
}

public fun merge_coins<T>(
    account: &mut VortexAccount,
    coins: vector<Receiving<Coin<T>>>,
    ctx: &mut TxContext,
) {
    transfer::public_transfer(account.merge(coins, ctx), account.id.to_address());
}

// === Package Functions ===

public(package) fun secret_hash(account: &VortexAccount): u256 {
    account.secret_hash
}

public(package) fun receive<T>(
    account: &mut VortexAccount,
    coins: vector<Receiving<Coin<T>>>,
    ctx: &mut TxContext,
): Coin<T> {
    account.merge(coins, ctx)
}

// === Private Functions ===

fun merge<T>(
    account: &mut VortexAccount,
    coins: vector<Receiving<Coin<T>>>,
    ctx: &mut TxContext,
): Coin<T> {
    coins.fold!(coin::zero<T>(ctx), |mut acc, coin| {
        acc.join(transfer::public_receive(&mut account.id, coin));

        acc
    })
}
