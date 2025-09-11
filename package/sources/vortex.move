module vortex::vortex;

use sui::{coin::Coin, sui::SUI};
use vortex::{vortex_config::VortexConfig, vortex_proof::Proof};

// === Public Functions ===

public fun deposit(
    config: &VortexConfig,
    proof: Proof,
    deposit: Coin<SUI>,
    encrypted_output: vector<u8>,
    ctx: &mut TxContext,
) {
    abort 0
}
