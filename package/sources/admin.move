module vortex::vortex_admin;

use interest_access_control::access_control;

// === Structs ===

public struct VORTEX_ADMIN() has drop;

// === Initializer ===

fun init(otw: VORTEX_ADMIN, ctx: &mut TxContext) {
    // Creates {ACL} shared object and {SuperAdmin} using the default ACL module
    // Source code: https://github.com/interest-protocol/interest-mvr/tree/main/access-control
    transfer::public_share_object(access_control::default(&otw, ctx));
}
