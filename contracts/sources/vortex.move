module vortex::vortex;

use interest_bps::bps::{Self, BPS};
use sui::{
    balance::{Self, Balance},
    coin::Coin,
    dynamic_object_field as dof,
    event::emit,
    groth16::{Self, Curve as Groth16Curve, PreparedVerifyingKey},
    sui::SUI,
    table::{Self, Table}
};
use vortex::{vortex_admin::VortexAdmin, vortex_merkle_tree::{Self, MerkleTree}};

// === Structs ===

public struct MerkleTreeKey() has copy, drop, store;

public struct Vortex has key {
    id: UID,
    deposit_fee: BPS,
    withdraw_fee: BPS,
    nullifier_hashes: Table<u256, bool>,
    groth16_vk: vector<u8>,
    groth16_curve: Groth16Curve,
    balance: Balance<SUI>,
    treasury: Balance<SUI>,
}

// === Mutative Functions ===

// === Public Views ===

public fun root(self: &Vortex): u256 {
    self.merkle_tree().root()
}

// === Admin Functions ===

public fun new(_: &VortexAdmin, ctx: &mut TxContext): Vortex {
    let merkle_tree = vortex_merkle_tree::new(ctx);

    let mut vortex = Vortex {
        id: object::new(ctx),
        deposit_fee: bps::new(0),
        withdraw_fee: bps::new(0),
        groth16_vk: vector[],
        groth16_curve: groth16::bn254(),
        nullifier_hashes: table::new(ctx),
        balance: balance::zero(),
        treasury: balance::zero(),
    };

    dof::add(&mut vortex.id, MerkleTreeKey(), merkle_tree);

    vortex
}

public fun share(self: Vortex) {
    transfer::share_object(self);
}

public fun set_deposit_fee(
    self: &mut Vortex,
    _: &VortexAdmin,
    fee_raw_value: u64,
    _ctx: &mut TxContext,
) {
    self.deposit_fee = bps::new(fee_raw_value);
}

public fun set_withdraw_fee(
    self: &mut Vortex,
    _: &VortexAdmin,
    fee_raw_value: u64,
    _ctx: &mut TxContext,
) {
    self.withdraw_fee = bps::new(fee_raw_value);
}

public fun set_groth16_vk(
    self: &mut Vortex,
    _: &VortexAdmin,
    vk: vector<u8>,
    _ctx: &mut TxContext,
) {
    self.groth16_vk = vk;
}

public fun collect_treasury(self: &mut Vortex, _: &VortexAdmin, ctx: &mut TxContext): Coin<SUI> {
    self.treasury.withdraw_all().into_coin(ctx)
}

// === Private Functions ===

fun assert_proof_vortex(self: &Vortex, vortex: address) {
    assert!(vortex == self.id.to_address(), vortex::vortex_errors::invalid_proof_vortex!());
}

fun assert_root_is_known(self: &Vortex, root: u256) {
    assert!(self.merkle_tree().is_known_root(root), vortex::vortex_errors::proof_root_not_known!());
}

fun verifying_key(self: &Vortex): PreparedVerifyingKey {
    groth16::prepare_verifying_key(
        &self.groth16_curve,
        &self.groth16_vk,
    )
}

fun take_fee(self: &mut Vortex, coin: &mut Balance<SUI>, bps: BPS): u64 {
    let fee_value = bps.calc_up(coin.value());

    self.treasury.join(coin.split(fee_value));

    fee_value
}

fun merkle_tree(self: &Vortex): &MerkleTree {
    dof::borrow(&self.id, MerkleTreeKey())
}

fun merkle_tree_mut(self: &mut Vortex): &mut MerkleTree {
    dof::borrow_mut(&mut self.id, MerkleTreeKey())
}
