module vortex::vortex_errors;

// === Package Functions ===

public(package) macro fun invalid_allowed_deposit_value(): u64 {
    1
}

public(package) macro fun merkle_tree_overflow(): u64 {
    2
}

public(package) macro fun proof_root_not_known(): u64 {
    3
}

public(package) macro fun invalid_poseidon_input(): u64 {
    4
}

public(package) macro fun invalid_proof(): u64 {
    5
}

public(package) macro fun invalid_proof_vortex(): u64 {
    6
}

public(package) macro fun invalid_address(): u64 {
    7
}

public(package) macro fun invalid_zero_value(): u64 {
    8
}