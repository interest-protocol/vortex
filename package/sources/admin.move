module vortex::vortex_admin;

// === Structs ===

public struct VortexAdmin has key, store {
    id: UID,
}

// === Initializer ===

fun init(ctx: &mut TxContext) {
    // Create a new VortexAdmin object and transfer it to the sender (module publisher)
    transfer::public_transfer(
        VortexAdmin {
            // Generate a new unique identifier for this admin object
            id: object::new(ctx),
        },
        // Transfer the admin object to the transaction sender (module publisher)
        ctx.sender(),
    );
}

// === Public Functions ===

public fun destroy(self: VortexAdmin) {
    let VortexAdmin { id } = self;

    id.delete();
}
