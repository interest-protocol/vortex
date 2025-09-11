module vortex::vortex_config;

use interest_bps::bps::{Self, BPS};
use sui::{balance::Balance, sui::SUI, vec_set::{Self, VecSet}};
use vortex::{vortex_admin::VortexAdmin, vortex_merkle_tree::VortexMerkleTree};

// === Structs ===

public struct VortexConfig has key {
    id: UID,
    deposit_fee: BPS,
    withdraw_fee: BPS,
    allowed_deposit_values: VecSet<u64>,
    merkle_tree: VortexMerkleTree,
}

// === Initializer ===

fun init(ctx: &mut TxContext) {
    // Creates the global configuration object with initial settings
    // Sets deposit and withdraw fees to 0% (no fees initially)
    let vortex_config = VortexConfig {
        id: object::new(ctx),
        deposit_fee: bps::new(0),
        withdraw_fee: bps::new(0),
        allowed_deposit_values: vec_set::empty(),
        merkle_tree: vortex::vortex_merkle_tree::new(ctx),
    };

    transfer::share_object(vortex_config);
}

// === Package Functions ===

public(package) fun assert_allowed_deposit_value(self: &VortexConfig, value: u64) {
    assert!(
        self.allowed_deposit_values.contains(&value),
        vortex::vortex_errors::invalid_allowed_deposit_value!(),
    );
}

public(package) fun take_deposit_fee(
    self: &VortexConfig,
    balance: &mut Balance<SUI>,
    ctx: &mut TxContext,
): u64 {
    let fee_value = self.deposit_fee.calc_up(balance.value());

    transfer::public_transfer(balance.split(fee_value).into_coin(ctx), @treasury);

    fee_value
}

public(package) fun take_withdraw_fee(
    self: &VortexConfig,
    balance: &mut Balance<SUI>,
    ctx: &mut TxContext,
): u64 {
    let fee_value = self.withdraw_fee.calc_up(balance.value());

    transfer::public_transfer(balance.split(fee_value).into_coin(ctx), @treasury);

    fee_value
}

// === Admin Functions ===

public fun set_deposit_fee(
    self: &mut VortexConfig,
    _: &VortexAdmin,
    fee_raw_value: u64,
    _ctx: &mut TxContext,
) {
    self.deposit_fee = bps::new(fee_raw_value);
}

public fun set_withdraw_fee(
    self: &mut VortexConfig,
    _: &VortexAdmin,
    fee_raw_value: u64,
    _ctx: &mut TxContext,
) {
    self.withdraw_fee = bps::new(fee_raw_value);
}

public fun add_allowed_deposit_value(
    self: &mut VortexConfig,
    _: &VortexAdmin,
    value: u64,
    _ctx: &mut TxContext,
) {
    self.allowed_deposit_values.insert(value);
}

public fun remove_allowed_deposit_value(
    self: &mut VortexConfig,
    _: &VortexAdmin,
    value: u64,
    _ctx: &mut TxContext,
) {
    self.allowed_deposit_values.remove(&value);
}
