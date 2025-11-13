module vortex::vortex;

use std::string::String;
use sui::{
    balance::{Self, Balance},
    coin::Coin,
    dynamic_object_field as dof,
    event::emit,
    groth16::{Self, Curve, PreparedVerifyingKey},
    sui::SUI,
    table::{Self, Table}
};
use vortex::{vortex_ext_data::ExtData, vortex_merkle_tree::{Self, MerkleTree}, vortex_proof::Proof};

// === Structs ===

public struct MerkleTreeKey() has copy, drop, store;

public struct Vortex has key {
    id: UID,
    curve: Curve,
    vk: PreparedVerifyingKey,
    balance: Balance<SUI>,
    nullifier_hashes: Table<u256, bool>,
}

public struct Registry has key {
    id: UID,
    encryption_keys: Table<address, String>,
}

// === Events ===

public struct NewCommitment has copy, drop {
    index: u64,
    commitment: u256,
    encrypted_output: u256,
}

public struct NullifierSpent(u256) has copy, drop;

public struct NewEncryptionKey(address, String) has copy, drop;

// === Initializer ===

fun init(ctx: &mut TxContext) {
    let curve = groth16::bn254();

    let mut vortex = Vortex {
        id: object::new(ctx),
        vk: groth16::prepare_verifying_key(&curve, &vortex::vortex_constants::verifying_key!()),
        curve,
        balance: balance::zero(),
        nullifier_hashes: table::new(ctx),
    };

    dof::add(&mut vortex.id, MerkleTreeKey(), vortex_merkle_tree::new(ctx));

    let registry = Registry {
        id: object::new(ctx),
        encryption_keys: table::new(ctx),
    };

    transfer::share_object(vortex);
    transfer::share_object(registry);
}

// === Mutative Functions ===

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

public fun transact(
    self: &mut Vortex,
    proof: Proof,
    ext_data: ExtData,
    deposit: Coin<SUI>,
    ctx: &mut TxContext,
) {
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
                &proof.public_inputs(),
                &proof.points(),
            ),
        vortex::vortex_errors::invalid_proof!(),
    );

    let ext_value = ext_data.value();
    let relayer_fee = ext_data.relayer_fee();
    let ext_value_is_non_zero = ext_value > 0;

    if (ext_data.value_sign() && ext_value_is_non_zero) {
        assert!(deposit.value() == ext_value, vortex::vortex_errors::invalid_deposit_value!());
    } else if (!ext_data.value_sign() && ext_value_is_non_zero) {
        transfer::public_transfer(
            self.balance.split(ext_value - relayer_fee).into_coin(ctx),
            ext_data.recipient(),
        );
    };

    self.balance.join(deposit.into_balance());

    proof.input_nullifiers().do!(|nullifier| {
        self.nullifier_hashes.add(nullifier, true);
        emit(NullifierSpent(nullifier));
    });

    let merkle_tree_mut = self.merkle_tree_mut();
    let commitments = proof.output_commitments();

    merkle_tree_mut.append_commitment(commitments[0], ext_data.encrypted_output1());

    merkle_tree_mut.append_commitment(commitments[1], ext_data.encrypted_output2());

    if (relayer_fee > 0 && ext_value_is_non_zero)
        transfer::public_transfer(
            self.balance.split(relayer_fee).into_coin(ctx),
            ext_data.relayer(),
        );
}

// === Public Views ===

public fun root(self: &Vortex): u256 {
    self.merkle_tree().root()
}

public fun is_nullifier_spent(self: &Vortex, nullifier: u256): bool {
    self.nullifier_hashes.contains(nullifier)
}

public fun next_index(self: &Vortex): u64 {
    self.merkle_tree().next_index()
}

public fun encryption_key(registry: &Registry, address: address): Option<String> {
    if (registry.encryption_keys.contains(address)) {
        option::some(registry.encryption_keys[address])
    } else {
        option::none()
    }
}

// === Private Functions ===

fun assert_ext_data_hash(ext_data: ExtData, ext_data_hash: vector<u8>) {
    assert!(ext_data.to_hash() == ext_data_hash, vortex::vortex_errors::invalid_ext_data_hash!());
}

fun assert_root_is_known(self: &Vortex, root: u256) {
    assert!(self.merkle_tree().is_known_root(root), vortex::vortex_errors::proof_root_not_known!());
}

fun assert_public_value(proof: Proof, ext_data: ExtData) {
    assert!(
        proof.public_value() == ext_data.public_value(),
        vortex::vortex_errors::invalid_public_value!(),
    );
}

fun append_commitment(tree: &mut MerkleTree, commitment: u256, encrypted_output: u256) {
    let index = tree.next_index();

    tree.append(commitment);

    emit(NewCommitment {
        commitment,
        index,
        encrypted_output,
    });
}

fun merkle_tree(self: &Vortex): &MerkleTree {
    dof::borrow(&self.id, MerkleTreeKey())
}

fun merkle_tree_mut(self: &mut Vortex): &mut MerkleTree {
    dof::borrow_mut(&mut self.id, MerkleTreeKey())
}

// === Aliases ===

use fun assert_ext_data_hash as ExtData.assert_hash;
use fun assert_public_value as Proof.assert_public_value;
use fun append_commitment as MerkleTree.append_commitment;
