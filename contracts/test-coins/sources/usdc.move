/// ! Important: This module is for testing purposes only.
module test_coins::usdc;

use sui::coin::{TreasuryCap, Coin};
use sui::coin_registry;

// === Structs ===

public struct USDC() has drop;

public struct Treasury has key {
    id: UID,
    treasury_cap: TreasuryCap<USDC>,
}

// === Initializer ===

fun init(otw: USDC, ctx: &mut TxContext) {
    let (currency_initializer, treasury_cap) = coin_registry::new_currency_with_otw(
        otw,
        6,
        b"USDC".to_string(),
        b"USDC".to_string(),
        b"Test USDC Coin for Vortex".to_string(),
        b"https://cryptologos.cc/logos/usd-coin-usdc-logo.png".to_string(),
        ctx,
    );

    let metadata_cap = currency_initializer.finalize(ctx);

    transfer::public_transfer(metadata_cap, ctx.sender());

    let treasury = Treasury {
        id: object::new(ctx),
        treasury_cap,
    };

    transfer::share_object(treasury);
}

// === Public Mutative Functions ===

public fun mint(self: &mut Treasury, value: u64, ctx: &mut TxContext): Coin<USDC> {
    self.treasury_cap.mint(value, ctx)
}
