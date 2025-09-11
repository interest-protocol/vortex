module vortex::vortex;

use sui::package;

// === Structs ===

public struct VORTEX() has drop;

// === Initializer ===

fun init(otw: VORTEX, ctx: &mut TxContext) {
    // Creates a {Publisher} and sends to the deployer.
    // This is to allow us to create Display objects.
    package::claim_and_keep(otw, ctx);
}

// === Public Functions === 