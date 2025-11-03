module vortex::vortex_admin;

// === Owned Objects ===

public struct VortexAdmin has key, store {
    id: UID,
}

// === Initializer ===

fun init(ctx: &mut TxContext) {
    transfer::public_transfer(
        VortexAdmin {
            id: object::new(ctx),
        },
        ctx.sender(),
    );
}

// === Public Mutative Functions ===

public fun destroy(self: VortexAdmin) {
    let VortexAdmin { id } = self;

    id.delete();
}
