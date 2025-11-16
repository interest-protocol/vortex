module vortex::vortex_proof;

use sui::groth16::{Self, PublicProofInputs, ProofPoints};

// === Structs ===

public struct Proof has copy, drop, store {
    root: u256,
    points: ProofPoints,
    input_nullifiers: vector<u256>,
    output_commitments: vector<u256>,
    public_value: u64,
    ext_data_hash: vector<u8>,
}

// === Public View Functions ===

public fun new(
    a: vector<u8>,
    b: vector<u8>,
    c: vector<u8>,
    root: u256,
    input_nullifier1: u256,
    input_nullifier2: u256,
    output_commitment1: u256,
    output_commitment2: u256,
    public_value: u64,
    ext_data_hash: vector<u8>,
): Proof {
    Proof {
        root,
        points: new_points(a, b, c),
        input_nullifiers: vector[input_nullifier1, input_nullifier2],
        output_commitments: vector[output_commitment1, output_commitment2],
        public_value,
        ext_data_hash,
    }
}

// === Package View Functions ===

public(package) fun root(self: Proof): u256 {
    self.root
}

public(package) fun points(self: Proof): ProofPoints {
    self.points
}

public(package) fun input_nullifiers(self: Proof): vector<u256> {
    self.input_nullifiers
}

public(package) fun output_commitments(self: Proof): vector<u256> {
    self.output_commitments
}

public(package) fun public_value(self: Proof): u64 {
    self.public_value
}

public(package) fun ext_data_hash(self: Proof): vector<u8> {
    self.ext_data_hash
}

public(package) fun public_inputs(self: Proof): PublicProofInputs {
    let mut bytes = vector[];

    bytes.append(self.root.to_bytes());
    bytes.append((self.public_value as u256).to_bytes());
    bytes.append(self.ext_data_hash);
    bytes.append(self.input_nullifiers[0].to_bytes());
    bytes.append(self.input_nullifiers[1].to_bytes());
    bytes.append(self.output_commitments[0].to_bytes());
    bytes.append(self.output_commitments[1].to_bytes());

    groth16::public_proof_inputs_from_bytes(bytes)
}

// === Private Functions ===

fun new_points(a: vector<u8>, b: vector<u8>, c: vector<u8>): ProofPoints {
    // Handle both old format (single proof) and new format (A, B, C components)
    let points = if (b.length() == 0 && c.length() == 0) {
        // Old format: single proof string
        a
    } else {
        // New format: concatenate A, B, C components
        let mut bytes = vector[];
        bytes.append(a);
        bytes.append(b);
        bytes.append(c);
        bytes
    };

    groth16::proof_points_from_bytes(points)
}

// === Aliases ===

use fun vortex::vortex_utils::u256_to_bytes as u256.to_bytes;