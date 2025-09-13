module vortex::vortex_utils;

use std::bcs::to_bytes;

// Convert any value to a 32-byte field element suitable for BN254/Poseidon
public(package) macro fun to_field_element_bytes<$T>($value: $T): vector<u8> {
    let value = $value;
    let value_bytes = to_bytes(&value);

    // First convert to u256 to check against field modulus
    let mut field_value = bytes_to_u256_le!(value_bytes);

    // Ensure the value is within the field (reduce modulo the prime)
    if (
        field_value >= 21888242871839275222246405745257275088548364400416034343698204186575808495617
    ) {
        field_value =
            field_value % 21888242871839275222246405745257275088548364400416034343698204186575808495617;
    };

    // Convert back to 32-byte little-endian representation
    u256_to_32_bytes_le!(field_value)
}

// Helper function to convert bytes to u256 (assumes big-endian input)
public(package) macro fun bytes_to_u256_le($bytes: vector<u8>): u256 {
    let bytes = $bytes;
    let mut result: u256 = 0;
    let mut i = 0;
    let len = bytes.length();

    while (i < len && i < 32) {
        // Max 32 bytes for u256
        let byte_value = (*vector::borrow(&bytes, i) as u256);
        result = result + (byte_value << (8 * (i as u8))); // Little-endian: LSB first
        i = i + 1;
    };

    result
}

// Helper function to convert u256 to 32-byte big-endian
public(package) macro fun u256_to_32_bytes_le($value: u256): vector<u8> {
    let value = $value;
    let mut bytes = vector[];
    let mut temp = value;

    // Extract bytes in little-endian order (LSB first)
    let mut i = 0;
    while (i < 32) {
        bytes.push_back((temp & 0xFF) as u8);
        temp = temp >> 8;
        i = i + 1;
    };

    bytes
}
