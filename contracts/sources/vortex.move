module vortex::vortex;

use sui::{
    balance::{Self, Balance},
    coin::{Self, Coin},
    dynamic_object_field as dof,
    event::emit,
    groth16::{Self, PreparedVerifyingKey},
    sui::SUI,
    table::{Self, Table}
};
use vortex::{vortex_ext_data::ExtData, vortex_merkle_tree::{Self, MerkleTree}, vortex_proof::Proof};

// === Structs ===

public struct MerkleTreeKey() has copy, drop, store;

public struct InitCap has key {
    id: UID,
}

public struct Vortex has key {
    id: UID,
    nullifier_hashes: Table<u256, bool>,
    vk: PreparedVerifyingKey,
    balance: Balance<SUI>,
}

// === Events ===

public struct NewCommitment has copy, drop {
    commitment: u256,
    index: u64,
    encrypted_output: u256,
}

// === Initializer ===

fun init(ctx: &mut TxContext) {
    let init_cap = InitCap {
        id: object::new(ctx),
    };

    transfer::transfer(init_cap, ctx.sender());
}

// === Mutative Functions ===

public fun new(init_cap: InitCap, vk: vector<u8>, ctx: &mut TxContext): Vortex {
    let InitCap { id } = init_cap;

    id.delete();

    let merkle_tree = vortex_merkle_tree::new(ctx);

    let mut vortex = Vortex {
        id: object::new(ctx),
        vk: groth16::prepare_verifying_key(
            &groth16::bn254(),
            &vk,
        ),
        nullifier_hashes: table::new(ctx),
        balance: balance::zero(),
    };

    dof::add(&mut vortex.id, MerkleTreeKey(), merkle_tree);

    vortex
}

public fun share(self: Vortex) {
    transfer::share_object(self);
}

public fun transact(
    self: &mut Vortex,
    proof: Proof,
    ext_data: ExtData,
    deposit: Coin<SUI>,
    ctx: &mut TxContext,
): Coin<SUI> {
    self.assert_root_is_known(proof.root());

    ext_data.assert_hash(proof.ext_data_hash());

    proof.assert_public_amount(ext_data);

    proof.input_nullifiers().do!(|nullifier| {
        assert!(
            !self.nullifier_hashes.contains(nullifier),
            vortex::vortex_errors::nullifier_already_spent!(),
        );
    });

    assert!(
        groth16::verify_groth16_proof(
            &groth16::bn254(),
            &self.vk,
            &proof.public_inputs(),
            &proof.points(),
        ),
        vortex::vortex_errors::invalid_proof!(),
    );

    let ext_value = ext_data.value();
    let relayer_fee = ext_data.relayer_fee();

    if (ext_data.value_sign() && ext_value > 0) {
        assert!(deposit.value() == ext_value, vortex::vortex_errors::invalid_deposit_value!());
    } else if (!ext_data.value_sign() && ext_value > 0) {
        transfer::public_transfer(
            self.balance.split(ext_value - relayer_fee).into_coin(ctx),
            ext_data.recipient(),
        );
    };

    self.balance.join(deposit.into_balance());

    let nex_index_to_insert = self.merkle_tree().next_index();
    let merkle_tree_mut = self.merkle_tree_mut();
    let commitments = proof.output_commitments();

    merkle_tree_mut.append(commitments[0]);
    merkle_tree_mut.append(commitments[1]);

    let second_index = nex_index_to_insert + 1;

    let relayer_fee_coin = if (relayer_fee > 0) {
        assert!(ctx.sender() == ext_data.relayer(), vortex::vortex_errors::invalid_relayer!());
        self.balance.split(relayer_fee).into_coin(ctx)
    } else {
        coin::zero(ctx)
    };

    proof.input_nullifiers().do!(|nullifier| {
        self.nullifier_hashes.add(nullifier, true);
    });

    emit(NewCommitment {
        commitment: commitments[0],
        index: nex_index_to_insert,
        encrypted_output: ext_data.encrypted_output1(),
    });

    emit(NewCommitment {
        commitment: commitments[1],
        index: second_index,
        encrypted_output: ext_data.encrypted_output2(),
    });

    relayer_fee_coin
}

// === Public Views ===

public fun root(self: &Vortex): u256 {
    self.merkle_tree().root()
}

// === Private Functions ===

fun assert_ext_data_hash(ext_data: ExtData, ext_data_hash: vector<u8>) {
    assert!(ext_data.to_hash() == ext_data_hash, vortex::vortex_errors::invalid_ext_data_hash!());
}

fun assert_root_is_known(self: &Vortex, root: u256) {
    assert!(self.merkle_tree().is_known_root(root), vortex::vortex_errors::proof_root_not_known!());
}

fun assert_public_amount(proof: Proof, ext_data: ExtData) {
    assert!(
        proof.public_value() == ext_data.public_amount(),
        vortex::vortex_errors::invalid_public_amount!(),
    );
}

fun merkle_tree(self: &Vortex): &MerkleTree {
    dof::borrow(&self.id, MerkleTreeKey())
}

fun merkle_tree_mut(self: &mut Vortex): &mut MerkleTree {
    dof::borrow_mut(&mut self.id, MerkleTreeKey())
}

// === Aliases ===

use fun assert_ext_data_hash as ExtData.assert_hash;
use fun assert_public_amount as Proof.assert_public_amount;
