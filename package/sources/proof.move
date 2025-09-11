module vortex::vortex_proof;

// === Structs ===

public struct Tuple has copy, drop, store {
    first: vector<u8>,
    second: vector<u8>,
}

public struct Proof has copy, drop, store {
    a: vector<u8>,
    b: vector<u8>,
    c: vector<u8>,
    root: vector<u8>,
    public_amount: vector<u8>,
    ext_data_hash: vector<u8>,
    input_nullifiers: Tuple,
    output_commitments: Tuple,
}

// === Public Functions ===

public fun new(
    a: vector<u8>,
    b: vector<u8>,
    c: vector<u8>,
    root: vector<u8>,
    public_amount: vector<u8>,
    ext_data_hash: vector<u8>,
    input_nullifiers: vector<vector<u8>>,
    output_commitments: vector<vector<u8>>,
): Proof {
    assert!(a.length() == 64, vortex::vortex_errors::invalid_proof_a!());

    assert!(b.length() == 128, vortex::vortex_errors::invalid_proof_b!());

    assert!(c.length() == 64, vortex::vortex_errors::invalid_proof_c!());

    assert!(root.length() == 32, vortex::vortex_errors::invalid_proof_root!());

    assert!(public_amount.length() == 32, vortex::vortex_errors::invalid_proof_public_amount!());

    assert!(ext_data_hash.length() == 32, vortex::vortex_errors::invalid_proof_ext_data_hash!());

    assert!(
        input_nullifiers.length() == 2,
        vortex::vortex_errors::invalid_proof_input_nullifiers!(),
    );

    assert!(
        input_nullifiers[0].length() == 32,
        vortex::vortex_errors::invalid_proof_input_nullifiers!(),
    );

    assert!(
        input_nullifiers[1].length() == 32,
        vortex::vortex_errors::invalid_proof_input_nullifiers!(),
    );

    assert!(
        output_commitments.length() == 2,
        vortex::vortex_errors::invalid_proof_output_commitments!(),
    );

    assert!(
        output_commitments[0].length() == 32,
        vortex::vortex_errors::invalid_proof_output_commitments!(),
    );

    assert!(
        output_commitments[1].length() == 32,
        vortex::vortex_errors::invalid_proof_output_commitments!(),
    );

    Proof {
        a,
        b,
        c,
        root,
        public_amount,
        ext_data_hash,
        input_nullifiers: input_nullifiers.to_tuple(),
        output_commitments: output_commitments.to_tuple(),
    }
}

// === Package Functions ===

fun to_tuple(x: vector<vector<u8>>): Tuple {
    Tuple { first: x[0], second: x[1] }
}

// === Aliases

use fun to_tuple as vector.to_tuple;
