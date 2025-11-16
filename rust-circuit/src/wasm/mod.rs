use crate::{
    circuit::TransactionCircuit,
    constants::LEVEL,
    merkle_tree::Path,
    poseidon::{poseidon_bn254, PoseidonHash},
};
use ark_bn254::{Bn254, Fr};
use ark_crypto_primitives::snark::SNARK;
use ark_groth16::Groth16;
use ark_serialize::{CanonicalDeserialize, CanonicalSerialize};
use num_bigint::BigUint;
use num_traits::Num;
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use wasm_bindgen::prelude::*;

// Set panic hook for better error messages in browser
#[wasm_bindgen(start)]
pub fn main() {
    console_error_panic_hook::set_once();
}

/// Proof output structure that matches the expected format for Sui Move contracts
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProofOutput {
    /// Proof component A (compressed: 32 bytes)
    pub proof_a: Vec<u8>,
    /// Proof component B (compressed: 64 bytes)  
    pub proof_b: Vec<u8>,
    /// Proof component C (compressed: 32 bytes)
    pub proof_c: Vec<u8>,
    /// All public inputs in order expected by Move contract
    pub public_inputs: Vec<String>,
}

/// Input structure for proof generation
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProofInput {
    // Public inputs
    pub root: String,
    pub public_amount: String,
    pub ext_data_hash: String,
    pub input_nullifier_1: String,
    pub input_nullifier_2: String,
    pub output_commitment_1: String,
    pub output_commitment_2: String,

    // Private inputs - Input UTXOs
    pub in_private_key_1: String,
    pub in_private_key_2: String,
    pub in_amount_1: String,
    pub in_amount_2: String,
    pub in_blinding_1: String,
    pub in_blinding_2: String,
    pub in_path_index_1: String,
    pub in_path_index_2: String,

    // Merkle paths (array of [left, right] pairs for each level)
    pub merkle_path_1: Vec<[String; 2]>,
    pub merkle_path_2: Vec<[String; 2]>,

    // Private inputs - Output UTXOs
    pub out_public_key_1: String,
    pub out_public_key_2: String,
    pub out_amount_1: String,
    pub out_amount_2: String,
    pub out_blinding_1: String,
    pub out_blinding_2: String,
}

/// Generates a zero-knowledge proof for a privacy-preserving transaction
///
/// # Arguments
/// * `input_json` - JSON string containing all circuit inputs
/// * `proving_key_hex` - Hex-encoded proving key (generated during setup)
///
/// # Returns
/// JSON string containing the proof and public inputs
///
/// # Example
/// ```javascript
/// const input = {
///   root: "12345...",
///   publicAmount: "1000",
///   // ... other inputs
/// };
/// const proof = prove(JSON.stringify(input), provingKeyHex);
/// const { proofA, proofB, proofC, publicInputs } = JSON.parse(proof);
/// ```
#[wasm_bindgen]
pub fn prove(input_json: &str, proving_key_hex: &str) -> Result<String, JsValue> {
    // Parse input
    let input: ProofInput = serde_json::from_str(input_json)
        .map_err(|e| JsValue::from_str(&format!("Failed to parse input JSON: {}", e)))?;

    // Parse proving key
    let pk_bytes = hex::decode(proving_key_hex)
        .map_err(|e| JsValue::from_str(&format!("Failed to decode proving key hex: {}", e)))?;

    let pk = ark_groth16::ProvingKey::<Bn254>::deserialize_compressed(&pk_bytes[..])
        .map_err(|e| JsValue::from_str(&format!("Failed to deserialize proving key: {}", e)))?;

    // Convert input strings to field elements
    let root = parse_field_element(&input.root)?;
    let public_amount = parse_field_element(&input.public_amount)?;
    let ext_data_hash = parse_field_element(&input.ext_data_hash)?;

    let input_nullifiers = [
        parse_field_element(&input.input_nullifier_1)?,
        parse_field_element(&input.input_nullifier_2)?,
    ];

    let output_commitment = [
        parse_field_element(&input.output_commitment_1)?,
        parse_field_element(&input.output_commitment_2)?,
    ];

    let in_private_keys = [
        parse_field_element(&input.in_private_key_1)?,
        parse_field_element(&input.in_private_key_2)?,
    ];

    let in_amounts = [
        parse_field_element(&input.in_amount_1)?,
        parse_field_element(&input.in_amount_2)?,
    ];

    let in_blindings = [
        parse_field_element(&input.in_blinding_1)?,
        parse_field_element(&input.in_blinding_2)?,
    ];

    let in_path_indices = [
        parse_field_element(&input.in_path_index_1)?,
        parse_field_element(&input.in_path_index_2)?,
    ];

    // Parse Merkle paths
    let merkle_paths = [
        parse_merkle_path(&input.merkle_path_1)?,
        parse_merkle_path(&input.merkle_path_2)?,
    ];

    let out_public_keys = [
        parse_field_element(&input.out_public_key_1)?,
        parse_field_element(&input.out_public_key_2)?,
    ];

    let out_amounts = [
        parse_field_element(&input.out_amount_1)?,
        parse_field_element(&input.out_amount_2)?,
    ];

    let out_blindings = [
        parse_field_element(&input.out_blinding_1)?,
        parse_field_element(&input.out_blinding_2)?,
    ];

    // Create circuit
    let poseidon_config = poseidon_bn254();
    let hasher = PoseidonHash::new(poseidon_config);

    let circuit = TransactionCircuit::new(
        hasher,
        root,
        public_amount,
        ext_data_hash,
        input_nullifiers,
        output_commitment,
        in_private_keys,
        in_amounts,
        in_blindings,
        in_path_indices,
        merkle_paths,
        out_public_keys,
        out_amounts,
        out_blindings,
    )
    .map_err(|e| JsValue::from_str(&format!("Failed to create circuit: {}", e)))?;

    // Generate proof using deterministic RNG for testing
    // In production, you should use a secure RNG
    use rand_chacha::ChaCha20Rng;
    use rand_core::SeedableRng;

    let mut rng = ChaCha20Rng::from_entropy();

    let proof = Groth16::<Bn254>::prove(&pk, circuit, &mut rng)
        .map_err(|e| JsValue::from_str(&format!("Failed to generate proof: {}", e)))?;

    // Serialize proof components (compressed format)
    let mut proof_a_bytes = Vec::new();
    proof
        .a
        .serialize_compressed(&mut proof_a_bytes)
        .map_err(|e| JsValue::from_str(&format!("Failed to serialize proof.a: {}", e)))?;

    let mut proof_b_bytes = Vec::new();
    proof
        .b
        .serialize_compressed(&mut proof_b_bytes)
        .map_err(|e| JsValue::from_str(&format!("Failed to serialize proof.b: {}", e)))?;

    let mut proof_c_bytes = Vec::new();
    proof
        .c
        .serialize_compressed(&mut proof_c_bytes)
        .map_err(|e| JsValue::from_str(&format!("Failed to serialize proof.c: {}", e)))?;

    // Public inputs in the order expected by Move contract
    let public_inputs = vec![
        root.to_string(),
        public_amount.to_string(),
        ext_data_hash.to_string(),
        input_nullifiers[0].to_string(),
        input_nullifiers[1].to_string(),
        output_commitment[0].to_string(),
        output_commitment[1].to_string(),
    ];

    let output = ProofOutput {
        proof_a: proof_a_bytes,
        proof_b: proof_b_bytes,
        proof_c: proof_c_bytes,
        public_inputs,
    };

    serde_json::to_string(&output)
        .map_err(|e| JsValue::from_str(&format!("Failed to serialize output: {}", e)))
}

/// Verifies a proof (useful for testing before submitting to chain)
///
/// # Arguments
/// * `proof_json` - JSON string containing proof output from `prove()`
/// * `verifying_key_hex` - Hex-encoded verifying key
///
/// # Returns
/// "true" if proof is valid, "false" otherwise
#[wasm_bindgen]
pub fn verify(proof_json: &str, verifying_key_hex: &str) -> Result<String, JsValue> {
    // Parse proof output
    let proof_output: ProofOutput = serde_json::from_str(proof_json)
        .map_err(|e| JsValue::from_str(&format!("Failed to parse proof JSON: {}", e)))?;

    // Parse verifying key
    let vk_bytes = hex::decode(verifying_key_hex)
        .map_err(|e| JsValue::from_str(&format!("Failed to decode verifying key hex: {}", e)))?;

    let vk = ark_groth16::VerifyingKey::<Bn254>::deserialize_compressed(&vk_bytes[..])
        .map_err(|e| JsValue::from_str(&format!("Failed to deserialize verifying key: {}", e)))?;

    // Deserialize proof components
    let proof_a = ark_bn254::G1Affine::deserialize_compressed(&proof_output.proof_a[..])
        .map_err(|e| JsValue::from_str(&format!("Failed to deserialize proof.a: {}", e)))?;

    let proof_b = ark_bn254::G2Affine::deserialize_compressed(&proof_output.proof_b[..])
        .map_err(|e| JsValue::from_str(&format!("Failed to deserialize proof.b: {}", e)))?;

    let proof_c = ark_bn254::G1Affine::deserialize_compressed(&proof_output.proof_c[..])
        .map_err(|e| JsValue::from_str(&format!("Failed to deserialize proof.c: {}", e)))?;

    let proof = ark_groth16::Proof {
        a: proof_a,
        b: proof_b,
        c: proof_c,
    };

    // Parse public inputs
    let public_inputs: Result<Vec<Fr>, JsValue> = proof_output
        .public_inputs
        .iter()
        .map(|s| parse_field_element(s))
        .collect();
    let public_inputs = public_inputs?;

    // Verify proof
    let pvk = ark_groth16::prepare_verifying_key(&vk);
    let is_valid = Groth16::<Bn254>::verify_proof(&pvk, &proof, &public_inputs)
        .map_err(|e| JsValue::from_str(&format!("Verification failed: {}", e)))?;

    Ok(is_valid.to_string())
}

// Helper functions
fn parse_field_element(s: &str) -> Result<Fr, JsValue> {
    // Handle both decimal and hex strings
    let s = s.trim();

    if s.starts_with("0x") || s.starts_with("0X") {
        // Hex format
        let big_uint = BigUint::from_str_radix(&s[2..], 16)
            .map_err(|e| JsValue::from_str(&format!("Failed to parse hex '{}': {}", s, e)))?;
        Ok(Fr::from(big_uint))
    } else {
        // Decimal format
        let big_uint = BigUint::from_str(s)
            .map_err(|e| JsValue::from_str(&format!("Failed to parse decimal '{}': {}", s, e)))?;
        Ok(Fr::from(big_uint))
    }
}

fn parse_merkle_path(path_data: &[[String; 2]]) -> Result<Path<LEVEL>, JsValue> {
    if path_data.len() != LEVEL {
        return Err(JsValue::from_str(&format!(
            "Invalid Merkle path length: expected {}, got {}",
            LEVEL,
            path_data.len()
        )));
    }

    let mut path = [(Fr::from(0u64), Fr::from(0u64)); LEVEL];

    for (i, pair) in path_data.iter().enumerate() {
        let left = parse_field_element(&pair[0])?;
        let right = parse_field_element(&pair[1])?;
        path[i] = (left, right);
    }

    Ok(Path { path })
}
