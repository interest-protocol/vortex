module vortex::vortex;

use interest_access_control::access_control;
use sui::package;

// === Structs ===

public struct VORTEX() has drop;
// === Initializer ===

fun init(otw: VORTEX, ctx: &mut TxContext) {
    // Creates {ACL} shared object and {SuperAdmin} using the default ACL module
    // Source code: https://github.com/interest-protocol/interest-mvr/tree/main/access-control
    transfer::public_share_object(access_control::default(&otw, ctx));

    // Creates a {Publisher} and sends to the deployer.
    // This is to allow us to create Display objects.
    package::claim_and_keep(otw, ctx);
}
