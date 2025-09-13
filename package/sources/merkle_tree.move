module vortex::vortex_merkle_tree;

use sui::{poseidon, table::{Self, Table}};

// === Constants ===

const HEIGHT: u64 = 26;

const ROOT_HISTORY_SIZE: u64 = 100;

const BN254_FIELD_MODULUS: u256 =
    21888242871839275222246405745257275088548364400416034343698204186575808495617;

// === Structs ===

public struct VortexMerkleTree has key, store {
    id: UID,
    next_index: u64,
    subtrees: vector<u256>,
    root_history: Table<u64, u256>,
    root_index: u64,
}

// === Package View Functions ===

public(package) fun root(self: &VortexMerkleTree): u256 {
    self.root_history[self.root_index]
}

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
            i = ROOT_HISTORY_SIZE - 1;
        } else {
            i = i - 1;
        };

        if (i == self.root_index) break;
    };

    false
}

// === Package Mutative Functions ===

public(package) fun new(ctx: &mut TxContext): VortexMerkleTree {
    let zeros_vector = zeros_vector!();

    let mut root_history = table::new(ctx);
    root_history.add(0, zeros_vector[HEIGHT - 1]);

    VortexMerkleTree {
        id: object::new(ctx),
        next_index: 0,
        subtrees: zeros_vector,
        root_history,
        root_index: 0,
    }
}

public(package) fun append(self: &mut VortexMerkleTree, leaf: u256): u64 {
    // Maximum capacity is 2^height leaves.
    assert!(
        1u64 << (HEIGHT as u8) > self.next_index,
        vortex::vortex_errors::merkle_tree_overflow!(),
    );

    leaf.is_valid_poseidon_input!();

    let mut current_index = self.next_index;
    let mut current_level_hash = leaf;
    let mut left: u256;
    let mut right: u256;
    let zeros_vector = zeros_vector!();

    HEIGHT.do!(|i| {
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

    self.next_index = self.next_index + 1;
    self.root_index = new_root_index;
    self.safe_history_add(new_root_index, current_level_hash);

    self.next_index - 1
}

// === Private Functions ===

fun safe_history_add(self: &mut VortexMerkleTree, index: u64, value: u256) {
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
    assert!($x < BN254_FIELD_MODULUS, vortex::vortex_errors::invalid_poseidon_input!());
}

macro fun zeros_vector(): vector<u256> {
    vector[
        // The zeros table is generated from scripts/src/make-zeros.ts using this TypeScript code:
        //
        // const ZERO_VALUE = poseidon1(stringToField('vortex'));
        // const zeros: bigint[] = [];
        //
        // let currentZero = ZERO_VALUE;
        // zeros.push(currentZero);
        //
        // for (let i = 1; i < treeLevels; i++) {
        //     currentZero = poseidon2(currentZero, currentZero); // hashLeftRight
        //     zeros.push(currentZero);
        // }
        //
        // This creates a hierarchical structure where:
        // - Level 0: ZERO_VALUE = Poseidon("vortex")
        // - Level 1: Poseidon(ZERO_VALUE, ZERO_VALUE)
        // - Level 2: Poseidon(Level1_zero, Level1_zero)
        // - Level i: Poseidon(Level(i-1)_zero, Level(i-1)_zero)
        //
        // Each level's zero value represents the hash of two zeros from the level below.
        // This ensures that when building Merkle proofs, we can use these precomputed
        // zero values as placeholders for empty subtrees, maintaining the tree structure
        // even when leaves haven't been inserted yet.
        18688842432741139442778047327644092677418528270738216181718229581494125774932u256,
        929670100605127589096201729966801143828059989180770638007278601230757123028u256,
        20059153686521406362481271315473498068253845102360114882796737328118528819600u256,
        667276972495892769517195136104358636854444397700904910347259067486374491460u256,
        12333205860481369973758777121486440301866097422034925170601892818077919669856u256,
        13265906118204670164732063746425660672195834675096811019428798251172285860978u256,
        3254533810100792365765975246297999341668420141674816325048742255119776645299u256,
        18309808253444361227126414342398728022042151803316641228967342967902364963927u256,
        12126650299593052178871547753567584772895820192048806970138326036720774331291u256,
        9949817351285988369728267498508465715570337443235086859122087250007803517342u256,
        11208526958197959509185914785003803401681281543885952782991980697855275912368u256,
        59685738145310886711325295148553591612803302297715439999772116453982910402u256,
        20837058910394942465479261789141487609029093821244922450759151002393360448717u256,
        8209451842087447702442792222326370366485985268583914555249981462794434142285u256,
        19651337661238139284113069695072175498780734789512991455990330919229086149402u256,
        11527931080332651861006914960138009072130600556413592683110711451245237795573u256,
        20764556403192106825184782309105498322242675071639346714780565918367449744227u256,
        10818178251908058160377157228631396071771716850372988172358158281935915764080u256,
        21598305620835755437985090087223184201582363356396834169567261294737143234327u256,
        16481295130402928965223624965091828506529631770925981912487987233811901391354u256,
        17911512007742433173433956238979622028159186641781974955249650899638270671335u256,
        5186032540459307640178997905000265487821097518169449170073506338735292796958u256,
        19685513117592528774434273738957742787082069361009067298107167967352389473358u256,
        10912258653908058948673432107359060806004349811796220228800269957283778663923u256,
        19880031465088514794850462701773174075421406509504511537647395867323147191667u256,
        18344394662872801094289264994998928886741543433797415760903591256277307773470u256,
    ]
}

// === Aliases ===

use fun assert_poseidon_input as u256.is_valid_poseidon_input;
