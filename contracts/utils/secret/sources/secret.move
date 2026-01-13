module secret::secret;

// === Structs ===

public struct Secret has key {
    id: UID,
    encrypted_secret: u256
}

// === Public Mutative Functions ===

public fun new(encrypted_secret: u256, ctx: &mut TxContext): Secret {
    Secret {
        id: object::new(ctx),
        encrypted_secret,
    }
}

public fun keep(secret: Secret, ctx: &TxContext) {
    transfer::transfer(secret, ctx.sender());
}

public fun delete(secret: Secret) {
    let  Secret { id, encrypted_secret: _ } = secret;
    
    id.delete();
}