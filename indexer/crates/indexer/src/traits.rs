use move_core_types::account_address::AccountAddress;
use serde::Serialize;

/// Trait for Move structs that can be matched against event types
pub trait MoveStruct: Serialize {
    const MODULE: &'static str;
    const NAME: &'static str;

    /// Check if a struct tag matches this event type for the given package
    fn matches_event_type(
        event_type: &move_core_types::language_storage::StructTag,
        package_address: &AccountAddress,
    ) -> bool {
        &event_type.address == package_address
            && event_type.module.as_str() == Self::MODULE
            && event_type.name.as_str() == Self::NAME
    }
}
