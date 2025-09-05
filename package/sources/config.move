module vortex::vortex_config;

use interest_access_control::access_control::AdminWitness;
use interest_bps::bps::{Self, BPS};
use sui::{
    balance::{Self, Balance},
    sui:: SUI,
    table::{Self, Table},
    vec_set::{Self, VecSet}
};
use vortex::vortex::VORTEX;

// === Structs ===

public struct VortexConfig has key {
    id: UID,
    deposit_fee: BPS,
    withdraw_fee: BPS,
    version: u64,
    allowed_deposit_values: VecSet<u64>,
    fee_balance: Balance<SUI>,
}

// === Initializer ===

fun init(ctx: &mut TxContext) {
    // Creates the global configuration object with initial settings
    // Sets deposit and withdraw fees to 0% (no fees initially)
    let vortex_config = VortexConfig {
        id: object::new(ctx),
        deposit_fee: bps::new(0),
        withdraw_fee: bps::new(0),
        version: vortex::vortex_constants::package_version!(),
        allowed_deposit_values: vec_set::empty(),
        fee_balance: balance::zero(),
    };

    transfer::share_object(vortex_config);
}

// === Package Functions ===

public(package) fun assert_package_version(self: &VortexConfig) {
    assert!(
        self.version == vortex::vortex_constants::package_version!(),
        vortex::vortex_errors::invalid_package_version!(),
    );
}

public(package) fun assert_allowed_deposit_value(self: &VortexConfig, value: u64) {
    assert!(
        self.allowed_deposit_values.contains(&value),
        vortex::vortex_errors::invalid_allowed_deposit_value!(),
    );
}

public(package) fun take_deposit_fee(self: &mut VortexConfig, balance: &mut Balance<SUI>): u64 {
    let fee_value = self.deposit_fee.calc_up(balance.value());

    self.fee_balance.join(balance.split(fee_value));

    fee_value
}

public(package) fun take_withdraw_fee(self: &mut VortexConfig, balance: &mut Balance<SUI>): u64 {
    let fee_value = self.withdraw_fee.calc_up(balance.value());

    self.fee_balance.join(balance.split(fee_value));

    fee_value
}

// === Admin Only Functions ===

public fun set_deposit_fee(
    self: &mut VortexConfig,
    _: &AdminWitness<VORTEX>,
    fee_raw_value: u64,
    _ctx: &mut TxContext,
) {
    self.deposit_fee = bps::new(fee_raw_value);
}

public fun set_withdraw_fee(
    self: &mut VortexConfig,
    _: &AdminWitness<VORTEX>,
    fee_raw_value: u64,
    _ctx: &mut TxContext,
) {
    self.withdraw_fee = bps::new(fee_raw_value);
}

public fun set_version(
    self: &mut VortexConfig,
    _: &AdminWitness<VORTEX>,
    version: u64,
    _ctx: &mut TxContext,
) {
    self.version = version;
}

public fun add_allowed_deposit_value(
    self: &mut VortexConfig,
    _: &AdminWitness<VORTEX>,
    value: u64,
    _ctx: &mut TxContext,
) {
    self.allowed_deposit_values.insert(value);
}

public fun remove_allowed_deposit_value(
    self: &mut VortexConfig,
    _: &AdminWitness<VORTEX>,
    value: u64,
    _ctx: &mut TxContext,
) {
    self.allowed_deposit_values.remove(&value);
}
