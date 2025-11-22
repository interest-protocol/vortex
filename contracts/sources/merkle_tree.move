module vortex::vortex_merkle_tree;

use std::u64;
use sui::{poseidon, table::{Self, Table}};

// === Constants ===

const HEIGHT: u64 = 26;
const ROOT_HISTORY_SIZE: u64 = 100;

// === Structs ===

public struct MerkleTree has key, store {
    id: UID,
    next_index: u64,
    subtrees: vector<u256>,
    root_history: Table<u64, u256>,
    root_index: u64,
}

// === Package Mutative Functions ===

public(package) fun new(ctx: &mut TxContext): MerkleTree {
    let empty_subtree_hashes = vortex::vortex_constants::empty_subtree_hashes!();

    let mut root_history = table::new(ctx);
    root_history.add(0, empty_subtree_hashes[HEIGHT]);

    let mut subtrees = vector[];

    u64::range_do!(0, HEIGHT, |i| {
        subtrees.push_back(empty_subtree_hashes[i]);
    });

    MerkleTree {
        id: object::new(ctx),
        next_index: 0,
        subtrees,
        root_history,
        root_index: 0,
    }
}

// Append two commitments at once (matching Nova)
public(package) fun append_pair(self: &mut MerkleTree, commitment0: u256, commitment1: u256) {
    // Maximum capacity check
    assert!(
        (1u64 << (HEIGHT as u8)) > self.next_index,
        vortex::vortex_errors::merkle_tree_overflow!(),
    );

    // Start by hashing the two leaves together
    let mut current_index = self.next_index / 2;
    let mut current_level_hash = poseidon2(commitment0, commitment1);
    let empty_subtree_hashes = vortex::vortex_constants::empty_subtree_hashes!();

    // Process levels 1 to HEIGHT (exclusive) (matching Nova: for i = 1; i < levels)
    u64::range_do!(1, HEIGHT, |i| {
        let subtree = &mut self.subtrees[i];
        let mut left: u256;
        let mut right: u256;

        if (current_index % 2 == 0) {
            left = current_level_hash;
            right = empty_subtree_hashes[i];
            *subtree = current_level_hash;
        } else {
            left = *subtree;
            right = current_level_hash;
        };

        current_level_hash = poseidon2(left, right);
        current_index = current_index / 2;
    });

    // Update root history
    let new_root_index = (self.root_index + 1) % ROOT_HISTORY_SIZE;
    self.root_index = new_root_index;
    self.safe_history_add(new_root_index, current_level_hash);
    self.next_index = self.next_index + 2;
}

// === Package View Functions ===

public(package) fun root(self: &MerkleTree): u256 {
    self.root_history[self.root_index]
}

public(package) fun next_index(self: &MerkleTree): u64 {
    self.next_index
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
    let modulus = vortex::vortex_constants::bn254_field_modulus!();
    let a_reduced = a % modulus;
    let b_reduced = b % modulus;
    poseidon::poseidon_bn254(&vector[a_reduced, b_reduced])
}
