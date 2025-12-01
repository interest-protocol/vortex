module vortex::vortex_utils;

use sui::bcs;

// === Package View Functions ===

public(package) fun u256_to_bytes(value: u256): vector<u8> {
    bcs::to_bytes(&(value))
}

public(package) fun u256_to_field(value: u256): vector<u8> {
    u256_to_bytes(value % vortex::vortex_constants::bn254_field_modulus!())
}
