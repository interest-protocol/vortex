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

public(package) macro fun invalid_proof_a(): u64 {
    5
}

public(package) macro fun invalid_proof_b(): u64 {
    6
}

public(package) macro fun invalid_proof_c(): u64 {
    7
}

public(package) macro fun invalid_proof_root(): u64 {
    8
}

public(package) macro fun invalid_proof_public_amount(): u64 {
    9
}

public(package) macro fun invalid_proof_ext_data_hash(): u64 {  
    10
}

public(package) macro fun invalid_proof_input_nullifiers(): u64 {
    11
}

public(package) macro fun invalid_proof_output_commitments(): u64 {
    12
}