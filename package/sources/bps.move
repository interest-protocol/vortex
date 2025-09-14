/// https://github.com/interest-protocol/interest-mvr/blob/main/bps/sources/bps.move
module vortex::bps;

// === Errors ===

const EOverflow: u64 = 0;

// === Structs ===

/// A struct to represent a percentage in basis points (bps).
public struct BPS(u64) has copy, drop, store;

// === Public Mutative Functions ===

public fun new(bps: u64): BPS {
    BPS(assert_overflow(bps))
}

/// @total is a raw value, not a BPS value.
/// It rounds up to the nearest integer.
public fun calc_up(bps: BPS, total: u64): u64 {
    let (x, y, z) = (bps.0 as u128, total as u128, max_value!() as u128);

    let amount = ((x * y) / z) + if ((x * y) % z > 0) 1 else 0;

    amount as u64
}

// === Public View Functions ===

/// 1 bps = 0.01%
/// 10,000 bps = 100%
public macro fun max_value(): u64 {
    10_000
}

// === Private Functions ===

fun assert_overflow(value: u64): u64 {
    assert!(value <= max_value!(), EOverflow);
    value
}
