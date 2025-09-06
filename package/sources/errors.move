module vortex::vortex_errors;

// === Package Functions ===

public(package) macro fun invalid_package_version(): u64 {
    1
}

public(package) macro fun invalid_allowed_deposit_value(): u64 {
    2
}

public(package) macro fun invalid_allowed_sender(): u64 {
    3
}

public(package) macro fun merkle_tree_overflow(): u64 {
    4
}
