module vortex::vortex_ext_data;

// === Structs ===

public struct ExtData has copy, drop, store {
    vortex: address,
    recipient: address,
    value: u64,
    relayer: address,
    relayer_fee: u64,
    encrypted_output1: u256,
    encrypted_output2: u256, 
}