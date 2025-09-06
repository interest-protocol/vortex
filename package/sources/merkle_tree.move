module vortex::vortex_merkle_tree;

use sui::{poseidon, table::{Self, Table}};

// === Structs ===

public struct VortexMerkleTree has key {
    id: UID,
    height: u64,
    next_index: u64,
    root: u256,
    subtrees: vector<u256>,
    root_history: Table<u64, u256>,
    root_history_size: u64,
    root_index: u64,
}

// === Initializer ===

fun init(ctx: &mut TxContext) {
    let zero_table = vortex::vortex_constants::zeros_table!();
    let height = vortex::vortex_constants::merkle_tree_height!();

    let initial_root = zero_table[height];

    let mut root_history = table::new(ctx);

    root_history.add(0, initial_root);

    let vortex_merkle_tree = VortexMerkleTree {
        id: object::new(ctx),
        height: vortex::vortex_constants::merkle_tree_height!(),
        next_index: 0,
        root: initial_root,
        subtrees: vector::tabulate!(height, |index| zero_table[index]),
        root_history,
        root_history_size: vortex::vortex_constants::merkle_tree_root_history_size!(),
        root_index: 0,
    };

    transfer::share_object(vortex_merkle_tree);
}

// === Public Functions ===

public fun share(self: VortexMerkleTree) {
    transfer::share_object(self);
}

// === Package Only Functions ===

public(package) fun is_known_root(self: &VortexMerkleTree, root: u256): bool {
    if (root == 0) return false;

    let mut i = self.root_index;

    loop {
        if (self.root_history.contains(i)) {
            if (self.root_history[i] == root) {
                return true
            };
        };

        if (i == 0) {
            i = self.root_history_size - 1;
        } else {
            i = i - 1;
        };

        if (i == self.root_index) break;
    };

    false
}

public(package) fun append(self: &mut VortexMerkleTree, leaf: u256): vector<u256> {
    // Maximum capacity is 2^height leaves.
    assert!(
        1u64 << (self.height as u8) > self.next_index,
        vortex::vortex_errors::merkle_tree_overflow!(),
    );

    let mut current_index = self.next_index;
    let mut current_level_hash = leaf;
    let mut left: u256;
    let mut right: u256;
    let mut proof: vector<u256> = vector[];
    let zero_table = vortex::vortex_constants::zeros_table!();

    self.height.do!(|i| {
        let subtree = &mut self.subtrees[i];
        let zero_byte = zero_table[i];

        if (current_index % 2 == 0) {
            left = current_level_hash;
            right = zero_byte;

            *subtree = current_level_hash;
            proof.push_back(right);
        } else {
            left = *subtree;
            right = current_level_hash;
            proof.push_back(left);
        };

        current_level_hash = poseidon2(left, right);
        current_index = current_index / 2;
    });

    let new_root_index = (self.root_index + 1) % self.root_history_size;

    self.root = current_level_hash;
    self.next_index = self.next_index + 1;
    self.root_index = new_root_index;
    self.safe_history_add(new_root_index, current_level_hash);

    proof
}

// === Private Functions ===

fun safe_history_add(self: &mut VortexMerkleTree, index: u64, value: u256) {
    if (self.root_history.contains(index)) {
        let old_value = &mut self.root_history[index];

        *old_value = value;
    };

    self.root_history.add(index, value);
}

/// H(a,b) = Poseidon([a,b]) over BN254.
fun poseidon2(a: u256, b: u256): u256 {
    poseidon::poseidon_bn254(&vector[a, b])
}
