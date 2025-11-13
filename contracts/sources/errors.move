module vortex::vortex_errors;

// === Package View Functions ===

public(package) macro fun invalid_allowed_deposit_value(): u64 {
    0
}

public(package) macro fun merkle_tree_overflow(): u64 {
    1
}

public(package) macro fun proof_root_not_known(): u64 {
    2
}

public(package) macro fun invalid_poseidon_input(): u64 {
    3
}

public(package) macro fun invalid_proof(): u64 {
    4
}

public(package) macro fun invalid_proof_vortex(): u64 {
    5
}

public(package) macro fun invalid_address(): u64 {
    6
}

public(package) macro fun invalid_zero_value(): u64 {
    7
}

public(package) macro fun invalid_ext_data_hash(): u64 {
    8
}

public(package) macro fun invalid_ext_data_value(): u64 {
    9
}

public(package) macro fun invalid_public_value(): u64 {
    10
}

public(package) macro fun nullifier_already_spent(): u64 {
    11
}

public(package) macro fun invalid_deposit_value(): u64 {
    12
}

public(package) macro fun invalid_relayer(): u64 {
    13
}

public(package) macro fun key_already_registered(): u64 {
    14
}