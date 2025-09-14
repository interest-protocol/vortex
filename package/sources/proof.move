module vortex::vortex_proof;

use std::bcs;
use sui::groth16::{Self, PublicProofInputs, ProofPoints};

// === Structs ===

public struct Proof has copy, drop, store {
    root: u256,
    nullifier: u256,
    recipient: address,
    value: u64,
    points: ProofPoints,
    public_inputs: PublicProofInputs,
    relayer: address,
    relayer_fee: u64,
}

public fun new(
    a: vector<u8>,
    b: vector<u8>,
    c: vector<u8>,
    root: u256,
    nullifier: u256,
    recipient: address,
    value: u64,
    relayer: address,
    relayer_fee: u64,
): Proof {
    Proof {
        root,
        nullifier,
        recipient,
        value,
        points: new_points(a, b, c),
        public_inputs: new_public_inputs(root, nullifier, recipient, relayer, relayer_fee),
        relayer,
        relayer_fee,
    }
}

// === Package View Functions ===

public(package) fun recipient(self: Proof): address {
    self.recipient
}

public(package) fun value(self: Proof): u64 {
    self.value
}

public(package) fun relayer(self: Proof): address {
    self.relayer
}

public(package) fun relayer_fee(self: Proof): u64 {
    self.relayer_fee
}

public(package) fun root(self: Proof): u256 {
    self.root
}

public(package) fun nullifier(self: Proof): u256 {
    self.nullifier
}

public(package) fun points(self: Proof): ProofPoints {
    self.points
}

public(package) fun public_inputs(self: Proof): PublicProofInputs {
    self.public_inputs
}

// === Private Functions ===

fun address_to_field(address: address): vector<u8> {
    bcs::to_bytes(&(address.to_u256() % vortex::vortex_constants::bn254_field_modulus!()))
}

fun u256_to_field(value: u256): vector<u8> {
    bcs::to_bytes(&(value % vortex::vortex_constants::bn254_field_modulus!()))
}

fun new_points(a: vector<u8>, b: vector<u8>, c: vector<u8>): ProofPoints {
    let mut bytes = vector[];

    bytes.append(bcs::to_bytes(&a));
    bytes.append(bcs::to_bytes(&b));
    bytes.append(bcs::to_bytes(&c));

    groth16::proof_points_from_bytes(bytes)
}

fun new_public_inputs(
    root: u256,
    nullifier: u256,
    recipient: address,
    relayer: address,
    relayer_fee: u64,
): PublicProofInputs {
    let mut bytes = vector[];

    bytes.append(root.to_field());
    bytes.append(nullifier.to_field());
    bytes.append(recipient.to_field());
    bytes.append(relayer.to_field());
    bytes.append((relayer_fee as u256).to_field());

    groth16::public_proof_inputs_from_bytes(bytes)
}

// === Aliases ===

use fun u256_to_field as u256.to_field;
use fun address_to_field as address.to_field;

// === Tests ===

#[test]
fun test_public_inputs() {
    new(
        vector[],
        vector[],
        vector[],
        3093576600674025166632687611856035295983036479389107935500477543414283790352,
        0x26152c6bf202a36b6e53f123cd67a28bd947050ba259674bc21c733decbd6e39,
        @0x0db8426f6207d23dc75352be968894e986d156d017ba1a217fcb521effcde94f,
        100000000,
        @0x0db8426f6207d23dc75352be968894e986d156d017ba1a217fcb521effcde94f,
        1,
    );
}
