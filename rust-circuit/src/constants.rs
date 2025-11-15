/// Merkle tree height (supports 2^26 = 67,108,864 commitments)
pub const MERKLE_TREE_LEVEL: usize = 26;

/// Number of input UTXOs per transaction
pub const N_INS: usize = 2;

/// Number of output UTXOs per transaction
pub const N_OUTS: usize = 2;

/// Maximum bits for amounts to prevent overflow (BN254 has 254 bits, we use 248)
pub const MAX_AMOUNT_BITS: usize = 248;

// For compatibility
pub const LEVEL: usize = MERKLE_TREE_LEVEL;
