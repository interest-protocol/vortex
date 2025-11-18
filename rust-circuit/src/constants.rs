/// Merkle tree height (supports 2^26 = 67,108,864 commitments)
///
/// This matches the Sui Move contract's MERKLE_TREE_HEIGHT constant.
/// Changing this requires redeploying contracts and regenerating keys.
pub const MERKLE_TREE_LEVEL: usize = 26;

/// Number of input UTXOs per transaction
///
/// Fixed at 2 for Vortex v1. Each input requires:
/// - Merkle proof verification (~6,500 constraints)
/// - Nullifier computation
/// - Range check
pub const N_INS: usize = 2;

/// Number of output UTXOs per transaction
///
/// Fixed at 2 for Vortex v1. Each output requires:
/// - Commitment computation
/// - Range check
pub const N_OUTS: usize = 2;

/// Maximum bits for amounts to prevent overflow
///
/// BN254 field has 254 bits total. We reserve:
/// - 6 bits for safety margin to prevent overflow in additions
/// - This allows up to 2^248 - 1 ≈ 4.5 × 10^74 smallest units
/// - With 9 decimals (like SUI), max value = 4.5 × 10^65 SUI
///
/// The circuit enforces that all input and output amounts fit within
/// this range to prevent arithmetic overflow during sum(inputs) + public_amount.
pub const MAX_AMOUNT_BITS: usize = 248;
