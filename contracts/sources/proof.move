module vortex::vortex_proof;

use sui::groth16::{Self, PublicProofInputs, ProofPoints};

// === Structs ===

public struct Proof has copy, drop, store {
    root: u256,
    points: ProofPoints,
    input_nullifiers: vector<u256>,
    output_commitments: vector<u256>,
    public_value: u64,
    ext_data_hash: u256,
}

// === Public View Functions ===

public fun new(
    proof_points: vector<u8>,
    root: u256,
    public_value: u64,
    ext_data_hash: u256,
    input_nullifier0: u256,
    input_nullifier1: u256,
    output_commitment0: u256,
    output_commitment1: u256,
): Proof {
    Proof {
        root,
        points: groth16::proof_points_from_bytes(proof_points),
        input_nullifiers: vector[input_nullifier0, input_nullifier1],
        output_commitments: vector[output_commitment0, output_commitment1],
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

public(package) fun ext_data_hash(self: Proof): u256 {
    self.ext_data_hash
}

public(package) fun public_inputs(self: Proof): PublicProofInputs {
    let bytes = vector[
        self.root.to_field(),
        // u64 is smaller than the field modulus of bn254, so we can use it directly
        (self.public_value as u256),
        self.ext_data_hash.to_field(),
        self.input_nullifiers[0].to_field(),
        self.input_nullifiers[1].to_field(),
        self.output_commitments[0].to_field(),
        self.output_commitments[1].to_field(),
    ];

   

    groth16::public_proof_inputs_from_bytes(bytes.to_bytes())
}

// === Aliases ===

use fun vortex::vortex_utils::u256_to_field as u256.to_field;
use fun vortex::vortex_utils::vector_u256_to_bytes as vector.to_bytes;
