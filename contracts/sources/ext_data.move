module vortex::vortex_ext_data;

use sui::hash;

// === Structs ===

public struct ExtData has copy, drop, store {
    vortex: address,
    recipient: address,
    value: u64,
    value_sign: bool,
    relayer: address,
    relayer_fee: u64,
    encrypted_output1: u256,
    encrypted_output2: u256,
}

// === Public Mutative Functions ===

public fun new(
    vortex: address,
    recipient: address,
    value: u64,
    value_sign: bool,
    relayer: address,
    relayer_fee: u64,
    encrypted_output1: u256,
    encrypted_output2: u256,
): ExtData {
    vortex.validate!();
    recipient.validate!();
    relayer.validate!();
    value.validate!();
    relayer_fee.validate!();

    ExtData {
        vortex,
        recipient,
        value_sign,
        value,
        relayer,
        relayer_fee,
        encrypted_output1,
        encrypted_output2,
    }
}

// === Package View Functions ===

public(package) fun vortex(self: ExtData): address {
    self.vortex
}

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

public(package) fun encrypted_output1(self: ExtData): u256 {
    self.encrypted_output1
}

public(package) fun encrypted_output2(self: ExtData): u256 {
    self.encrypted_output2
}

public(package) fun to_hash(self: ExtData): vector<u8> {
    let mut data = vector[];

    data.append(self.vortex.to_bytes());
    data.append(self.recipient.to_bytes());
    data.append(self.value.to_bytes());
    data.append(self.value_sign.to_bytes());
    data.append(self.relayer.to_bytes());
    data.append(self.relayer_fee.to_bytes());
    data.append(self.encrypted_output1.to_bytes());
    data.append(self.encrypted_output2.to_bytes());

    hash::blake2b256(&data)
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
