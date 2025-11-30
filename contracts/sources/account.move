module vortex::vortex_account;

use sui::{coin::{Self, Coin}, transfer::Receiving};

// === Structs ===

public struct VortexAccount has key, store {
    id: UID,
    hashed_secret: u256,
}

// === Public Mutative Functions ===

public fun new(hashed_secret: u256, ctx: &mut TxContext): VortexAccount {
    assert!(hashed_secret != 0, vortex::vortex_errors::invalid_hashed_secret!());

    let account = VortexAccount {
        id: object::new(ctx),
        hashed_secret,
    };

    vortex::vortex_events::new_account(account.id.to_address(), hashed_secret);

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

public(package) fun hashed_secret(account: &VortexAccount): u256 {
    account.hashed_secret
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
