module vortex::vortex_ext_data;

use sui::hash;

// === Structs ===

public struct ExtData has copy, drop, store {
    recipient: address,
    value: u64,
    value_sign: bool,
    relayer: address,
    relayer_fee: u64,
    encrypted_output0: vector<u8>,
    encrypted_output1: vector<u8>,
}

// === Public Mutative Functions ===

public fun new(
    recipient: address,
    value: u64,
    value_sign: bool,
    relayer: address,
    relayer_fee: u64,
    encrypted_output0: vector<u8>,
    encrypted_output1: vector<u8>,
): ExtData {
    recipient.validate!();
    value.validate!();

    assert!(
        vortex::vortex_constants::one_sui_in_mist!() >= value,
        vortex::vortex_errors::invalid_ext_data_value!(),
    );

    ExtData {
        recipient,
        value_sign,
        value,
        relayer,
        relayer_fee,
        encrypted_output0,
        encrypted_output1,
    }
}

// === Package View Functions ===

public(package) fun recipient(self: ExtData): address {
    self.recipient
}

public(package) fun value(self: ExtData): u64 {
    self.value
}

public(package) fun value_sign(self: ExtData): bool {
    self.value_sign
}

public(package) fun relayer(self: ExtData): address {
    self.relayer
}

public(package) fun relayer_fee(self: ExtData): u64 {
    self.relayer_fee
}

public(package) fun encrypted_output0(self: ExtData): vector<u8> {
    self.encrypted_output0
}

public(package) fun encrypted_output1(self: ExtData): vector<u8> {
    self.encrypted_output1
}

public(package) fun to_hash(self: ExtData): vector<u8> {
    let data = vector[
        self.recipient.to_bytes(),
        self.value.to_bytes(),
        self.value_sign.to_bytes(),
        self.relayer.to_bytes(),
        self.relayer_fee.to_bytes(),
        self.encrypted_output0.to_bytes(),
        self.encrypted_output1.to_bytes(),
    ];

    hash::blake2b256(&data.flatten()).to_bytes().skip(1)
}

public(package) fun public_value(ext_data: ExtData): u256 {
    let value = ext_data.value();
    let relayer_fee = ext_data.relayer_fee();

    if (ext_data.value_sign()) {
        // If it is a deposit, the pool should get value - fee.
        (value - relayer_fee) as u256
    } else {
        // If it is a withdrawal, the pool should get value + fee.
        vortex::vortex_constants::bn254_field_modulus!() - ((value + relayer_fee) as u256)
    }
}

// === Private Functions ===

macro fun assert_no_zero_address($address: address) {
    assert!($address != @0x0, vortex::vortex_errors::invalid_address!());
}

macro fun assert_no_zero_value($value: u64) {
    assert!($value != 0, vortex::vortex_errors::invalid_zero_value!());
}

// === Aliases ===

use fun assert_no_zero_value as u64.validate;
use fun assert_no_zero_address as address.validate;
use fun vortex::vortex_utils::u64_to_bytes as u64.to_bytes;
use fun vortex::vortex_utils::u256_to_bytes as u256.to_bytes;
use fun vortex::vortex_utils::bool_to_bytes as bool.to_bytes;
use fun vortex::vortex_utils::address_to_bytes as address.to_bytes;
use fun vortex::vortex_utils::vector_u8_to_bytes as vector.to_bytes;
