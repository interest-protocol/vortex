module vortex::vortex_ext_data;

// === Structs ===

public struct ExtData has copy, drop, store {
    value: u64,
    value_sign: bool,
    relayer: address,
    relayer_fee: u64,
    encrypted_output0: vector<u8>,
    encrypted_output1: vector<u8>,
}

// === Public Mutative Functions ===

public fun new(
    value: u64,
    value_sign: bool,
    relayer: address,
    relayer_fee: u64,
    encrypted_output0: vector<u8>,
    encrypted_output1: vector<u8>,
): ExtData {
    ExtData {
        value_sign,
        value,
        relayer,
        relayer_fee,
        encrypted_output0,
        encrypted_output1,
    }
}

// === Assert Functions ===

public(package) fun assert_relayer(self: ExtData, ctx: &TxContext) {
    if (self.relayer != @0x0) 
        assert!(self.relayer == ctx.sender(), vortex::vortex_errors::invalid_relayer!());
}

// === Package View Functions ===

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

public(package) fun public_value(ext_data: ExtData): u256 {
    if (ext_data.value_sign()) // If it is a deposit, the pool should get value - fee.
        (ext_data.value() - ext_data.relayer_fee()) as u256
    else // If it is a withdrawal, the pool should remove value
        vortex::vortex_constants::bn254_field_modulus!() - (ext_data.value() as u256)
}
