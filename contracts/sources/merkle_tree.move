module vortex::vortex_merkle_tree;

use std::u64;
use sui::{poseidon, table::{Self, Table}};

// === Constants ===

const HEIGHT: u64 = 31;

const ROOT_HISTORY_SIZE: u64 = 100;

// === Structs ===

public struct MerkleTree has key, store {
    id: UID,
    next_index: u64,
    subtrees: vector<u256>,
    root_history: Table<u64, u256>,
    root_index: u64,
}

// === Package View Functions ===

public(package) fun root(self: &MerkleTree): u256 {
    self.root_history[self.root_index]
}

public(package) fun is_known_root(self: &MerkleTree, root: u256): bool {
    if (root == 0) return false;

    let mut i = self.root_index;

    loop {
        if (self.root_history.contains(i)) {
            if (self.root_history[i] == root) {
                return true
            };
        };

        if (i == 0) {
            i = ROOT_HISTORY_SIZE - 1;
        } else {
            i = i - 1;
        };

        if (i == self.root_index) break;
    };

    false
}

// === Package Mutative Functions ===

public(package) fun new(ctx: &mut TxContext): MerkleTree {
    let zeros_vector = vortex::vortex_constants::zeros_vector!();

    let mut root_history = table::new(ctx);
    root_history.add(0, zeros_vector[HEIGHT]);

    MerkleTree {
        id: object::new(ctx),
        next_index: 0,
        subtrees: zeros_vector,
        root_history,
        root_index: 0,
    }
}

public(package) fun append(self: &mut MerkleTree, leaf1: u256, leaf2: u256): u64 {
    // Maximum capacity is 2^height leaves.
    assert!(
        1u64 << (HEIGHT as u8) > self.next_index,
        vortex::vortex_errors::merkle_tree_overflow!(),
    );

    leaf1.is_valid_poseidon_input!();
    leaf2.is_valid_poseidon_input!();

    let mut current_index = self.next_index / 2;
    let mut current_level_hash = poseidon2(leaf1, leaf2);
    let mut left: u256;
    let mut right: u256;
    let zeros_vector = vortex::vortex_constants::zeros_vector!();

    u64::range_do_eq!(1, HEIGHT, |i| {
        let subtree = &mut self.subtrees[i];

        if (current_index % 2 == 0) {
            left = current_level_hash;
            right = zeros_vector[i];

            *subtree = current_level_hash;
        } else {
            left = *subtree;
            right = current_level_hash;
        };

        current_level_hash = poseidon2(left, right);
        current_index = current_index / 2;
    });

    let new_root_index = (self.root_index + 1) % ROOT_HISTORY_SIZE;

    self.root_index = new_root_index;
    self.safe_history_add(new_root_index, current_level_hash);

    self.next_index = self.next_index + 2;

    self.next_index
}

// === Private Functions ===

fun safe_history_add(self: &mut MerkleTree, index: u64, value: u256) {
    if (self.root_history.contains(index)) {
        let old_value = &mut self.root_history[index];

        *old_value = value;
    } else {
        self.root_history.add(index, value);
    };
}

fun poseidon2(a: u256, b: u256): u256 {
    a.is_valid_poseidon_input!();
    b.is_valid_poseidon_input!();

    poseidon::poseidon_bn254(&vector[a, b])
}

macro fun assert_poseidon_input($x: u256) {
    assert!(
        $x < vortex::vortex_constants::bn254_field_modulus!(),
        vortex::vortex_errors::invalid_poseidon_input!(),
    );
}

// === Aliases ===

use fun assert_poseidon_input as u256.is_valid_poseidon_input;
