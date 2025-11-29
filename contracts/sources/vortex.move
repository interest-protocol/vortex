module vortex::vortex;

use std::{ascii::String, type_name};
use sui::{
    balance::{Self, Balance},
    coin::Coin,
    dynamic_object_field as dof,
    event::emit,
    groth16::{Self, Curve, PreparedVerifyingKey, PublicProofInputs},
    table::{Self, Table},
    transfer::Receiving
};
use vortex::{
    vortex_account::VortexAccount,
    vortex_ext_data::ExtData,
    vortex_merkle_tree::{Self, MerkleTree},
    vortex_proof::Proof
};

// === Structs ===

public struct MerkleTreeKey() has copy, drop, store;

public struct Vortex<phantom CoinType> has key {
    id: UID,
    curve: Curve,
    vk: PreparedVerifyingKey,
    balance: Balance<CoinType>,
    nullifier_hashes: Table<u256, bool>,
}

public struct Registry has key {
    id: UID,
    pools: Table<String, address>,
    encryption_keys: Table<address, String>,
}

// === Events ===

public struct NewPool<phantom CoinType>(address) has copy, drop;

public struct NewCommitment<phantom CoinType> has copy, drop {
    index: u64,
    commitment: u256,
    encrypted_output: vector<u8>,
}

public struct NullifierSpent<phantom CoinType>(u256) has copy, drop;

public struct NewEncryptionKey(address, String) has copy, drop;

// === Initializer ===

fun init(ctx: &mut TxContext) {
    let registry = Registry {
        id: object::new(ctx),
        pools: table::new(ctx),
        encryption_keys: table::new(ctx),
    };

    transfer::share_object(registry);
}

// === Mutative Functions ===

public fun new<CoinType>(registry: &mut Registry, ctx: &mut TxContext): Vortex<CoinType> {
    let id = type_name::with_defining_ids<CoinType>().into_string();

    assert!(!registry.pools.contains(id), vortex::vortex_errors::pool_already_exists!());

    let curve = groth16::bn254();

    let mut vortex = Vortex {
        id: object::new(ctx),
        vk: groth16::prepare_verifying_key(&curve, &vortex::vortex_constants::verifying_key!()),
        curve,
        balance: balance::zero(),
        nullifier_hashes: table::new(ctx),
    };

    let vortex_address = vortex.id.to_address();

    registry.pools.add(id, vortex_address);

    dof::add(&mut vortex.id, MerkleTreeKey(), vortex_merkle_tree::new(ctx));

    emit(NewPool<CoinType>(vortex_address));

    vortex
}

public fun share<CoinType>(vortex: Vortex<CoinType>) {
    transfer::share_object(vortex);
}

public fun register(registry: &mut Registry, encryption_key: String, ctx: &mut TxContext) {
    let sender = ctx.sender();

    if (registry.encryption_keys.contains(sender)) {
        let existing_key = &mut registry.encryption_keys[sender];
        assert!(existing_key != &encryption_key, vortex::vortex_errors::key_already_registered!());
        *existing_key = encryption_key;
    } else {
        registry.encryption_keys.add(sender, encryption_key);
    };

    emit(NewEncryptionKey(sender, encryption_key));
}

public fun transact<CoinType>(
    self: &mut Vortex<CoinType>,
    deposit: Coin<CoinType>,
    proof: Proof<CoinType>,
    ext_data: ExtData,
    ctx: &mut TxContext,
) {
    self.process_transaction(deposit, proof.public_inputs(), proof, ext_data, ctx);
}

public fun transact_with_account<CoinType>(
    self: &mut Vortex<CoinType>,
    account: &mut VortexAccount,
    coins: vector<Receiving<Coin<CoinType>>>,
    proof: Proof<CoinType>,
    ext_data: ExtData,
    ctx: &mut TxContext,
) {
    let deposit = account.receive(coins, ctx);

    self.process_transaction(
        deposit,
        proof.tto_public_inputs(account.secret_hash()),
        proof,
        ext_data,
        ctx,
    );
}

// === Public Views ===

public fun root<CoinType>(self: &Vortex<CoinType>): u256 {
    self.merkle_tree().root()
}

public fun is_nullifier_spent<CoinType>(self: &Vortex<CoinType>, nullifier: u256): bool {
    self.nullifier_hashes.contains(nullifier)
}

public fun next_index<CoinType>(self: &Vortex<CoinType>): u64 {
    self.merkle_tree().next_index()
}

public fun encryption_key(registry: &Registry, address: address): Option<String> {
    if (registry.encryption_keys.contains(address)) {
        option::some(registry.encryption_keys[address])
    } else {
        option::none()
    }
}

public fun vortex_address<CoinType>(registry: &Registry): Option<address> {
    let id = type_name::with_defining_ids<CoinType>().into_string();

    if (registry.pools.contains(id)) {
        option::some(registry.pools[id])
    } else {
        option::none()
    }
}

// === Private Functions ===

fun assert_address<CoinType>(self: &Vortex<CoinType>, vortex: address) {
    assert!(vortex == self.id.to_address(), vortex::vortex_errors::invalid_vortex!());
}

fun assert_ext_data_hash(ext_data: ExtData, ext_data_hash: u256) {
    assert!(
        ext_data.to_hash() == ext_data_hash.to_bytes(),
        vortex::vortex_errors::invalid_ext_data_hash!(),
    );
}

fun assert_root_is_known<CoinType>(self: &Vortex<CoinType>, root: u256) {
    assert!(self.merkle_tree().is_known_root(root), vortex::vortex_errors::proof_root_not_known!());
}

fun assert_public_value<CoinType>(proof: Proof<CoinType>, ext_data: ExtData) {
    assert!(
        proof.public_value() == ext_data.public_value(),
        vortex::vortex_errors::invalid_public_value!(),
    );
}

fun process_transaction<CoinType>(
    self: &mut Vortex<CoinType>,
    deposit: Coin<CoinType>,
    public_inputs: PublicProofInputs,
    proof: Proof<CoinType>,
    ext_data: ExtData,
    ctx: &mut TxContext,
) {
    self.assert_address(proof.vortex());

    self.assert_root_is_known(proof.root());

    ext_data.assert_hash(proof.ext_data_hash());

    proof.assert_public_value(ext_data);

    proof.input_nullifiers().do!(|nullifier| {
        assert!(
            !self.is_nullifier_spent(nullifier),
            vortex::vortex_errors::nullifier_already_spent!(),
        );
    });

    assert!(
        self
            .curve
            .verify_groth16_proof(
                &self.vk,
                &public_inputs,
                &proof.points(),
            ),
        vortex::vortex_errors::invalid_proof!(),
    );

    let ext_value = ext_data.value();

    if (ext_data.value_sign() && ext_value > 0)
        assert!(
            deposit.value() == ext_value + ext_data.relayer_fee(),
            vortex::vortex_errors::invalid_deposit_value!(),
        );

    if (!ext_data.value_sign() && ext_value > 0)
        transfer::public_transfer(
            self.balance.split(ext_value).into_coin(ctx),
            ext_data.recipient(),
        );

    self.balance.join(deposit.into_balance());

    proof.input_nullifiers().do!(|nullifier| {
        self.nullifier_hashes.add(nullifier, true);
        emit(NullifierSpent<CoinType>(nullifier));
    });

    let merkle_tree_mut = self.merkle_tree_mut();
    let commitments = proof.output_commitments();

    let next_index = merkle_tree_mut.next_index();

    merkle_tree_mut.append_pair(commitments[0], commitments[1]);

    emit(NewCommitment<CoinType> {
        index: next_index,
        commitment: commitments[0],
        encrypted_output: ext_data.encrypted_output0(),
    });

    emit(NewCommitment<CoinType> {
        index: next_index + 1,
        commitment: commitments[1],
        encrypted_output: ext_data.encrypted_output1(),
    });

    if (ext_data.relayer_fee() > 0)
        transfer::public_transfer(
            self.balance.split(ext_data.relayer_fee()).into_coin(ctx),
            ext_data.relayer(),
        );
}

fun merkle_tree<CoinType>(self: &Vortex<CoinType>): &MerkleTree {
    dof::borrow(&self.id, MerkleTreeKey())
}

fun merkle_tree_mut<CoinType>(self: &mut Vortex<CoinType>): &mut MerkleTree {
    dof::borrow_mut(&mut self.id, MerkleTreeKey())
}

// === Aliases ===

use fun assert_ext_data_hash as ExtData.assert_hash;
use fun assert_public_value as Proof.assert_public_value;
use fun vortex::vortex_utils::u256_to_bytes as u256.to_bytes;
