module vortex::vortex;

use interest_access_control::access_control::{Self, AdminWitness};
use interest_bps::bps::{Self, BPS};
use sui::{package, sui::{Self, SUI}, table::{Self, Table}, vec_set::{Self, VecSet}};

// === Structs ===

public struct VORTEX() has drop;

public struct GlobalConfig has key {
    id: UID,
    deposit_fee: BPS,
    withdraw_fee: BPS,
    version: u64,
}

// === Initializer ===

fun init(otw: VORTEX, ctx: &mut TxContext) {
    // Creates {ACL} shared object and {SuperAdmin} using the default ACL module
    // Source code: https://github.com/interest-protocol/interest-mvr/tree/main/access-control
    transfer::public_share_object(access_control::default(&otw, ctx));

    // Creates a {Publisher} and sends to the deployer.
    // This is to allow us to create Display objects.
    package::claim_and_keep(otw, ctx);

    // Creates the global configuration object with initial settings
    // Sets deposit and withdraw fees to 0% (no fees initially)
    let global_config = GlobalConfig {
        id: object::new(ctx),
        deposit_fee: bps::new(0),
        withdraw_fee: bps::new(0),
        version: vortex::vortex_constants::package_version(),
    };

    // Shares the global config as a shared object for public access
    transfer::share_object(global_config);
}

// === Admin Only Functions ===

public fun set_deposit_fee(
    config: &mut GlobalConfig,
    _: &AdminWitness<VORTEX>,
    fee_raw_value: u64,
    _ctx: &mut TxContext,
) {
    config.deposit_fee = bps::new(fee_raw_value);
}

public fun set_withdraw_fee(
    config: &mut GlobalConfig,
    _: &AdminWitness<VORTEX>,
    fee_raw_value: u64,
    _ctx: &mut TxContext,
) {
    config.withdraw_fee = bps::new(fee_raw_value);
}

public fun set_version(
    config: &mut GlobalConfig,
    _: &AdminWitness<VORTEX>,
    version: u64,
    _ctx: &mut TxContext,
) {
    config.version = version;
}
