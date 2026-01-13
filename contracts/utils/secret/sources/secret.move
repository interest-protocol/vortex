module secret::secret;

use std::ascii::String;

// === Structs ===

public struct Secret has key {
    id: UID,
    encrypted_secret: String,
}

// === Public Mutative Functions ===

public fun new(encrypted_secret: String, ctx: &mut TxContext): Secret {
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