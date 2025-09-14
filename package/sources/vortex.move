module vortex::vortex;

use sui::{
    balance::{Self, Balance},
    coin::Coin,
    dynamic_object_field as dof,
    event::emit,
    groth16::{Self, Curve as Groth16Curve, PreparedVerifyingKey},
    sui::SUI,
    table::{Self, Table}
};
use vortex::{
    bps::{Self, BPS},
    vortex_admin::VortexAdmin,
    vortex_merkle_tree::{Self, VortexMerkleTree},
    vortex_proof::Proof
};

// === Structs ===

public struct MerkleTreeKey() has copy, drop, store;

public struct Vortex has key {
    id: UID,
    deposit_fee: BPS,
    withdraw_fee: BPS,
    allowed_value: u64,
    nullifiers: Table<u256, bool>,
    commitments: Table<u256, bool>,
    groth16_vk: vector<u8>,
    groth16_curve: Groth16Curve,
    balance: Balance<SUI>,
}

// === Events ===

public struct Deposit has copy, drop {
    root: u256,
    commitment: u256,
    index: u64,
    value: u64,
    fee: u64,
}

public struct Withdraw has copy, drop {
    root: u256,
    value: u64,
    recipient: address,
    fee: u64,
    relayer_fee: u64,
    relayer: address,
    nullifier: u256,
}

// === Public Functions ===

public fun deposit(
    self: &mut Vortex,
    mut deposit: Coin<SUI>,
    commitment: u256,
    ctx: &mut TxContext,
) {
    let (deposit_value, fee_value) = self.take_deposit_fee(&mut deposit, ctx);

    self.assert_allowed_deposit_value(deposit_value - fee_value);

    self.commitments.add(commitment, true);

    self.balance.join(deposit.into_balance());

    let merkle_tree = self.merkle_tree_mut();

    let index = merkle_tree.append(commitment);

    emit(Deposit {
        commitment,
        index,
        value: deposit_value,
        fee: fee_value,
        root: merkle_tree.root(),
    });
}

public fun withdraw(self: &mut Vortex, proof: Proof, ctx: &mut TxContext) {
    self.assert_root_is_known(proof.root());
    self.assert_allowed_deposit_value(proof.value());

    self.nullifiers.add(proof.nullifier(), true);

    assert!(
        groth16::verify_groth16_proof(
            &self.groth16_curve,
            &self.verifying_key(),
            &proof.public_inputs(),
            &proof.points(),
        ),
        vortex::vortex_errors::invalid_proof!(),
    );

    let mut withdraw = self.balance.split(proof.value()).into_coin(ctx);

    let fee = self.take_withdraw_fee(&mut withdraw, ctx);

    let relayer_fee = withdraw.split(proof.relayer_fee(), ctx);

    emit(Withdraw {
        value: proof.value(),
        fee,
        relayer_fee: relayer_fee.value(),
        recipient: proof.recipient(),
        nullifier: proof.nullifier(),
        root: proof.root(),
        relayer: proof.relayer(),
    });

    transfer::public_transfer(relayer_fee, proof.relayer());

    transfer::public_transfer(withdraw, proof.recipient());
}

// === Admin Functions ===

public fun new(_: &VortexAdmin, value: u64, ctx: &mut TxContext): Vortex {
    let merkle_tree = vortex_merkle_tree::new(ctx);

    let mut vortex = Vortex {
        id: object::new(ctx),
        deposit_fee: bps::new(0),
        withdraw_fee: bps::new(0),
        allowed_value: value,
        groth16_vk: vector[],
        groth16_curve: groth16::bn254(),
        nullifiers: table::new(ctx),
        commitments: table::new(ctx),
        balance: balance::zero(),
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

// === Private Functions ===

fun assert_allowed_deposit_value(self: &Vortex, value: u64) {
    assert!(value == self.allowed_value, vortex::vortex_errors::invalid_allowed_deposit_value!());
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

fun take_deposit_fee(self: &Vortex, deposit: &mut Coin<SUI>, ctx: &mut TxContext): (u64, u64) {
    let deposit_value = deposit.value();

    let fee_value = self.deposit_fee.calc_up(deposit_value);

    transfer::public_transfer(deposit.split(fee_value, ctx), @treasury);

    (deposit_value, fee_value)
}

fun take_withdraw_fee(self: &Vortex, withdraw: &mut Coin<SUI>, ctx: &mut TxContext): u64 {
    let fee_value = self.withdraw_fee.calc_up(withdraw.value());

    transfer::public_transfer(withdraw.split(fee_value, ctx), @treasury);

    fee_value
}

fun merkle_tree(self: &Vortex): &VortexMerkleTree {
    dof::borrow(&self.id, MerkleTreeKey())
}

fun merkle_tree_mut(self: &mut Vortex): &mut VortexMerkleTree {
    dof::borrow_mut(&mut self.id, MerkleTreeKey())
}
