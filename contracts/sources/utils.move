module vortex::vortex_utils;

use sui::bcs;

// === Package View Functions ===

public(package) fun u256_to_bytes(value: u256): vector<u8> {
    bcs::to_bytes(&(value))
}

public(package) fun address_to_bytes(address: address): vector<u8> {
    bcs::to_bytes(&(address))
}

public(package) fun u64_to_bytes(value: u64): vector<u8> {
    bcs::to_bytes(&(value))
}

public(package) fun bool_to_bytes(value: bool): vector<u8> {
    bcs::to_bytes(&(value))
}

public(package) fun vector_u8_to_bytes(value: vector<u8>): vector<u8> {
    bcs::to_bytes(&(value))
}

public(package) fun address_to_field(address: address): vector<u8> {
    u256_to_field(address.to_u256())
}

public(package) fun u256_to_field(value: u256): vector<u8> {
    u256_to_bytes(value % vortex::vortex_constants::bn254_field_modulus!())
}

