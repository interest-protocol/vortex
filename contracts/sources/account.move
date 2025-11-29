module vortex::vortex_account;

use sui::{coin::{Self, Coin}, event::emit, transfer::Receiving};

// === Structs ===

public struct VortexAccount has key, store {
    id: UID,
    secret_hash: u256,
}

// === Events ===

public struct NewAccount(address, u256) has copy, drop;

// === Public Mutative Functions ===

public fun new(secret_hash: u256, ctx: &mut TxContext): VortexAccount {
    let account = VortexAccount {
        id: object::new(ctx),
        secret_hash,
    };

    emit(NewAccount(account.id.to_address(), secret_hash));

    account
}

public fun merge_coins<CoinType>(
    account: &mut VortexAccount,
    coins: vector<Receiving<Coin<CoinType>>>,
    ctx: &mut TxContext,
) {
    transfer::public_transfer(account.merge(coins, ctx), account.id.to_address());
}

// === Package Functions ===

public(package) fun secret_hash(account: &VortexAccount): u256 {
    account.secret_hash
}

public(package) fun receive<CoinType>(
    account: &mut VortexAccount,
    coins: vector<Receiving<Coin<CoinType>>>,
    ctx: &mut TxContext,
): Coin<CoinType> {
    account.merge(coins, ctx)
}

// === Private Functions ===

fun merge<CoinType>(
    account: &mut VortexAccount,
    coins: vector<Receiving<Coin<CoinType>>>,
    ctx: &mut TxContext,
): Coin<CoinType> {
    coins.fold!(coin::zero<CoinType>(ctx), |mut acc, coin| {
        acc.join(transfer::public_receive(&mut account.id, coin));

        acc
    })
}
