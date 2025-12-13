module vortex_swap::vortex_swap;

use std::{ascii::String, type_name};
use sui::{coin::{Self, Coin}, event::emit};
use vortex::{vortex::Vortex, vortex_ext_data::ExtData, vortex_proof::Proof};

// === Structs ===

public struct Receipt<phantom CoinIn, phantom CoinOut> {
    amount_in: u64,
    min_amount_out: u64,
    relayer: address,
    relayer_fee: u64,
}

// === Events ===

public struct SwapEvent has copy, drop {
    coin_in: String,
    coin_out: String,
    amount_in: u64,
    amount_out: u64,
    relayer: address,
    relayer_fee: u64,
}

// === Errors ===

#[error(code = 0)]
const EInvalidAmountOut: vector<u8> = b"Invalid amount out";

#[error(code = 1)]
const EUnauthorizedRelayer: vector<u8> = b"Unauthorized relayer";

// === Public Mutative Functions ===

public fun start_swap<CoinIn, CoinOut>(
    vortex: &mut Vortex<CoinIn>,
    proof: Proof<CoinIn>,
    ext_data: ExtData,
    relayer: address,
    relayer_fee: u64,
    min_amount_out: u64,
    ctx: &mut TxContext,
): (Receipt<CoinIn, CoinOut>, Coin<CoinIn>) {
    let coin_in = vortex.transact(coin::zero(ctx), proof, ext_data, ctx);

    let receipt = Receipt<CoinIn, CoinOut> {
        amount_in: coin_in.value(),
        min_amount_out,
        relayer,
        relayer_fee,
    };

    (receipt, coin_in)
}

public fun finish_swap<CoinIn, CoinOut>(
    vortex: &mut Vortex<CoinOut>,
    mut coin_out: Coin<CoinOut>,
    receipt: Receipt<CoinIn, CoinOut>,
    proof: Proof<CoinOut>,
    ext_data: ExtData,
    ctx: &mut TxContext,
) {
    let Receipt { amount_in, min_amount_out, relayer, relayer_fee } = receipt;

    let amount_out = coin_out.value();

    assert!(amount_out >= min_amount_out, EInvalidAmountOut);
    assert!(relayer == ctx.sender(), EUnauthorizedRelayer);

    transfer::public_transfer(
        coin_out.split(relayer_fee + amount_out.diff(min_amount_out), ctx),
        relayer,
    );

    vortex.transact(coin_out, proof, ext_data, ctx).destroy_zero();

    emit(SwapEvent {
        coin_in: type_name::with_defining_ids<CoinIn>().into_string(),
        coin_out: type_name::with_defining_ids<CoinOut>().into_string(),
        amount_in,
        amount_out: min_amount_out,
        relayer,
        relayer_fee: relayer_fee + amount_out.diff(min_amount_out),
    });
}
