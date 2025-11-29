module vortex::vortex_proof;

use sui::{bcs, groth16::{Self, PublicProofInputs, ProofPoints}};

// === Structs ===

public struct Proof<phantom CoinType> has copy, drop, store {
    root: u256,
    points: ProofPoints,
    input_nullifiers: vector<u256>,
    output_commitments: vector<u256>,
    public_value: u256,
    ext_data_hash: u256,
    vortex: address,
}

// === Public View Functions ===

public fun new<CoinType>(
    vortex: address,
    proof_points: vector<u8>,
    root: u256,
    public_value: u256,
    ext_data_hash: u256,
    input_nullifier0: u256,
    input_nullifier1: u256,
    output_commitment0: u256,
    output_commitment1: u256,
): Proof<CoinType> {
    Proof {
        root,
        points: groth16::proof_points_from_bytes(proof_points),
        input_nullifiers: vector[input_nullifier0, input_nullifier1],
        output_commitments: vector[output_commitment0, output_commitment1],
        public_value,
        ext_data_hash,
        vortex,
    }
}

// === Package View Functions ===

public(package) fun root<CoinType>(self: Proof<CoinType>): u256 {
    self.root
}

public(package) fun points<CoinType>(self: Proof<CoinType>): ProofPoints {
    self.points
}

public(package) fun input_nullifiers<CoinType>(self: Proof<CoinType>): vector<u256> {
    self.input_nullifiers
}

public(package) fun output_commitments<CoinType>(self: Proof<CoinType>): vector<u256> {
    self.output_commitments
}

public(package) fun public_value<CoinType>(self: Proof<CoinType>): u256 {
    self.public_value
}

public(package) fun ext_data_hash<CoinType>(self: Proof<CoinType>): u256 {
    self.ext_data_hash
}

public(package) fun vortex<CoinType>(self: Proof<CoinType>): address {
    self.vortex
}

public(package) fun public_inputs<CoinType>(self: Proof<CoinType>): PublicProofInputs {
    let bytes = vector[
        self.vortex.to_u256().to_field(),
        self.root.to_field(),
        self.public_value.to_field(),
        self.ext_data_hash.to_field(),
        self.input_nullifiers[0].to_field(),
        self.input_nullifiers[1].to_field(),
        self.output_commitments[0].to_field(),
        self.output_commitments[1].to_field(),
        zero_field!(),
        zero_field!(),
    ];

    groth16::public_proof_inputs_from_bytes(bytes.flatten())
}

public(package) fun tto_public_inputs<CoinType>(
    self: Proof<CoinType>,
    hashed_secret: u256,
): PublicProofInputs {
    let bytes = vector[
        self.vortex.to_u256().to_field(),
        self.root.to_field(),
        self.public_value.to_field(),
        self.ext_data_hash.to_field(),
        self.input_nullifiers[0].to_field(),
        self.input_nullifiers[1].to_field(),
        self.output_commitments[0].to_field(),
        self.output_commitments[1].to_field(),
        bcs::to_bytes(&1u256),
        hashed_secret.to_field(),
    ];

    groth16::public_proof_inputs_from_bytes(bytes.flatten())
}

// === Private Functions ===

macro fun zero_field(): vector<u8> {
    bcs::to_bytes(&0u256)
}

// === Aliases ===

use fun vortex::vortex_utils::u256_to_field as u256.to_field;
