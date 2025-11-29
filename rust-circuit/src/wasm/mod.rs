// use crate::{circuit::TransactionCircuit, constants::MERKLE_TREE_LEVEL, merkle_tree::Path};
// use ark_bn254::{Bn254, Fr};
// use ark_crypto_primitives::snark::SNARK;
// use ark_ff::PrimeField;
// use ark_groth16::Groth16;
// use ark_relations::r1cs::{ConstraintSynthesizer, ConstraintSystem};
// use ark_serialize::{CanonicalDeserialize, CanonicalSerialize};
// use num_bigint::BigUint;
// use serde::{Deserialize, Serialize};
// use std::str::FromStr;
// use wasm_bindgen::prelude::*;

// // Set panic hook for better error messages in browser
// #[wasm_bindgen(start)]
// pub fn main() {
//     console_error_panic_hook::set_once();
// }

// /// Proof output structure that matches the expected format for Sui Move contracts
// #[derive(Debug, Clone, Serialize, Deserialize)]
// #[serde(rename_all = "camelCase")]
// pub struct ProofOutput {
//     /// Proof component A (compressed: 32 bytes)
//     pub proof_a: Vec<u8>,
//     /// Proof component B (compressed: 64 bytes)
//     pub proof_b: Vec<u8>,
//     /// Proof component C (compressed: 32 bytes)
//     pub proof_c: Vec<u8>,
//     /// All public inputs in order expected by Move contract
//     pub public_inputs: Vec<String>,
//     pub proof_serialized_hex: String,
//     pub public_inputs_serialized_hex: String,
// }

// /// Input structure for proof generation
// #[derive(Debug, Clone, Deserialize)]
// #[serde(rename_all = "camelCase")]
// pub struct ProofInput {
//     // Public inputs
//     pub root: String,
//     pub public_amount: String,
//     pub ext_data_hash: String,
//     pub input_nullifier_0: String,
//     pub input_nullifier_1: String,
//     pub output_commitment_0: String,
//     pub output_commitment_1: String,

//     // Private inputs - Input UTXOs
//     pub in_private_key_0: String,
//     pub in_private_key_1: String,
//     pub in_amount_0: String,
//     pub in_amount_1: String,
//     pub in_blinding_0: String,
//     pub in_blinding_1: String,
//     pub in_path_index_0: String,
//     pub in_path_index_1: String,

//     // Merkle paths (array of [left, right] pairs for each level)
//     pub merkle_path_0: Vec<[String; 2]>,
//     pub merkle_path_1: Vec<[String; 2]>,

//     // Private inputs - Output UTXOs
//     pub out_public_key_0: String,
//     pub out_public_key_1: String,
//     pub out_amount_0: String,
//     pub out_amount_1: String,
//     pub out_blinding_0: String,
//     pub out_blinding_1: String,
// }

// /// Generates a zero-knowledge proof for a privacy-preserving transaction
// ///
// /// # Arguments
// /// * `input_json` - JSON string containing all circuit inputs
// /// * `proving_key_hex` - Hex-encoded proving key (generated during setup)
// ///
// /// # Returns
// /// JSON string containing the proof and public inputs
// ///
// /// # Example
// /// ```javascript
// /// const input = {
// ///   root: "12345...",
// ///   publicAmount: "1000",
// ///   // ... other inputs
// /// };
// /// const proof = prove(JSON.stringify(input), provingKeyHex);
// /// const { proofA, proofB, proofC, publicInputs } = JSON.parse(proof);
// /// ```
// #[wasm_bindgen]
// pub fn prove(input_json: &str, proving_key_hex: &str) -> Result<String, JsValue> {
//     // Parse input
//     let input: ProofInput = serde_json::from_str(input_json)
//         .map_err(|e| JsValue::from(&format!("Failed to parse input JSON: {}", e)))?;

//     // Parse proving key
//     let pk_bytes = hex::decode(proving_key_hex)
//         .map_err(|e| JsValue::from(&format!("Failed to decode proving key hex: {}", e)))?;

//     let pk = ark_groth16::ProvingKey::<Bn254>::deserialize_compressed(&pk_bytes[..])
//         .map_err(|e| JsValue::from(&format!("Failed to deserialize proving key: {}", e)))?;

//     // Convert input strings to field elements
//     let root = parse_field_element(&input.root)?;
//     let public_amount = parse_field_element(&input.public_amount)?;
//     let ext_data_hash = parse_field_element(&input.ext_data_hash)?;
//     let input_nullifier_0 = parse_field_element(&input.input_nullifier_0)?;
//     let input_nullifier_1 = parse_field_element(&input.input_nullifier_1)?;
//     let output_commitment_0 = parse_field_element(&input.output_commitment_0)?;
//     let output_commitment_1 = parse_field_element(&input.output_commitment_1)?;

//     let in_private_keys = [
//         parse_field_element(&input.in_private_key_0)?,
//         parse_field_element(&input.in_private_key_1)?,
//     ];

//     let in_amounts = [
//         parse_field_element(&input.in_amount_0)?,
//         parse_field_element(&input.in_amount_1)?,
//     ];

//     let in_blindings = [
//         parse_field_element(&input.in_blinding_0)?,
//         parse_field_element(&input.in_blinding_1)?,
//     ];

//     let in_path_indices = [
//         parse_field_element(&input.in_path_index_0)?,
//         parse_field_element(&input.in_path_index_1)?,
//     ];

//     // Parse Merkle paths
//     let merkle_paths = [
//         parse_merkle_path(&input.merkle_path_0)?,
//         parse_merkle_path(&input.merkle_path_1)?,
//     ];

//     let out_public_keys = [
//         parse_field_element(&input.out_public_key_0)?,
//         parse_field_element(&input.out_public_key_1)?,
//     ];

//     let out_amounts = [
//         parse_field_element(&input.out_amount_0)?,
//         parse_field_element(&input.out_amount_1)?,
//     ];

//     let out_blindings = [
//         parse_field_element(&input.out_blinding_0)?,
//         parse_field_element(&input.out_blinding_1)?,
//     ];

//     // Create circuit
//     let circuit = TransactionCircuit::new(
//         root,
//         public_amount,
//         ext_data_hash,
//         input_nullifier_0,
//         input_nullifier_1,
//         output_commitment_0,
//         output_commitment_1,
//         in_private_keys,
//         in_amounts,
//         in_blindings,
//         in_path_indices,
//         merkle_paths,
//         out_public_keys,
//         out_amounts,
//         out_blindings,
//     )
//     .map_err(|e| JsValue::from(&format!("Failed to create circuit: {}", e)))?;

//     // Generate proof using deterministic RNG for testing
//     // In production, you should use a secure RNG
//     use rand_chacha::ChaCha20Rng;
//     use rand_core::SeedableRng;

//     let mut rng = ChaCha20Rng::from_seed([0u8; 32]);

//     // Extract public inputs BEFORE proving (circuit is consumed by prove())
//     // The order MUST match the order in which FpVar::new_input() is called in generate_constraints()
//     // This is: root, public_amount, ext_data_hash, input_nullifier_0, input_nullifier_1,
//     //          output_commitment_0, output_commitment_1
//     let public_inputs_field = circuit.get_public_inputs();
//     let public_inputs_serialized = circuit
//         .get_public_inputs_serialized()
//         .map_err(|e| JsValue::from(&format!("Failed to serialize public inputs: {}", e)))?;

//     let cs = ConstraintSystem::<Fr>::new_ref();
//     circuit
//         .clone()
//         .generate_constraints(cs.clone())
//         .expect("Failed to generate constraints");
//     if !cs.is_satisfied().expect("Failed to check constraints") {
//         panic!("Constraints are not satisfied");
//     }

//     // Generate proof - Groth16 will internally call generate_constraints() and extract public inputs
//     // It uses the same public inputs we extracted above (in the same order)
//     // Note: Groth16's prove() function extracts public inputs from the constraint system
//     // in the order they were allocated via FpVar::new_input(). Our get_public_inputs()
//     // should match this order exactly.
//     //
//     // IMPORTANT: Groth16 extracts public inputs from the constraint system during prove().
//     // The public inputs are stored in the constraint system in the order they were allocated.
//     // We extract them manually using get_public_inputs() which should match exactly.
//     let proof = Groth16::<Bn254>::prove(&pk, circuit, &mut rng)
//         .map_err(|e| JsValue::from(&format!("Failed to generate proof: {}", e)))?;

//     // Serialize proof components (compressed format)
//     let mut proof_a_bytes = Vec::new();
//     proof
//         .a
//         .serialize_compressed(&mut proof_a_bytes)
//         .map_err(|e| JsValue::from(&format!("Failed to serialize proof.a: {}", e)))?;

//     let mut proof_b_bytes = Vec::new();
//     proof
//         .b
//         .serialize_compressed(&mut proof_b_bytes)
//         .map_err(|e| JsValue::from(&format!("Failed to serialize proof.b: {}", e)))?;

//     let mut proof_c_bytes = Vec::new();
//     proof
//         .c
//         .serialize_compressed(&mut proof_c_bytes)
//         .map_err(|e| JsValue::from(&format!("Failed to serialize proof.c: {}", e)))?;

//     // Serialize proof
//     let mut proof_serialized = Vec::new();
//     proof.serialize_compressed(&mut proof_serialized).unwrap();

//     // Convert public inputs to strings for JSON output
//     // Use the field's underlying representation for reliable serialization/deserialization
//     // This ensures the string can be parsed back correctly by parse_field_element()
//     let public_inputs: Vec<String> = public_inputs_field
//         .iter()
//         .map(|input| {
//             // Convert Fr to BigInt representation, then to string
//             // This ensures reliable round-trip conversion
//             input.into_bigint().to_string()
//         })
//         .collect();

//     let output = ProofOutput {
//         proof_a: proof_a_bytes,
//         proof_b: proof_b_bytes,
//         proof_c: proof_c_bytes,
//         public_inputs,
//         proof_serialized_hex: hex::encode(proof_serialized),
//         public_inputs_serialized_hex: hex::encode(public_inputs_serialized),
//     };

//     serde_json::to_string(&output)
//         .map_err(|e| JsValue::from(&format!("Failed to serialize output: {}", e)))
// }

// /// Verifies a proof (useful for testing before submitting to chain)
// ///
// /// # Arguments
// /// * `proof_json` - JSON string containing proof output from `prove()`
// /// * `verifying_key_hex` - Hex-encoded verifying key
// ///
// /// # Returns
// /// "true" if proof is valid, "false" otherwise
// #[wasm_bindgen]
// pub fn verify(proof_json: &str, verifying_key_hex: &str) -> Result<bool, JsValue> {
//     let proof_output: ProofOutput = serde_json::from_str(proof_json)
//         .map_err(|e| JsValue::from(&format!("Step 1 - Failed to parse proof JSON: {}", e)))?;

//     let vk_bytes = hex::decode(verifying_key_hex)
//         .map_err(|e| JsValue::from(&format!("Step 2 - Failed to decode VK hex: {}", e)))?;

//     let vk = ark_groth16::VerifyingKey::<Bn254>::deserialize_compressed(&vk_bytes[..])
//         .map_err(|e| JsValue::from(&format!("Step 3 - Failed to deserialize VK: {}", e)))?;

//     let pvk = ark_groth16::prepare_verifying_key(&vk);

//     let proof_bytes = hex::decode(&proof_output.proof_serialized_hex)
//         .map_err(|e| JsValue::from(&format!("Step 4 - Failed to decode proof hex: {}", e)))?;

//     let proof = ark_groth16::Proof::<Bn254>::deserialize_compressed(&proof_bytes[..])
//         .map_err(|e| JsValue::from(&format!("Step 5 - Failed to deserialize proof: {}", e)))?;

//     let public_inputs: Result<Vec<Fr>, JsValue> = proof_output
//         .public_inputs
//         .iter()
//         .enumerate()
//         .map(|(i, s)| {
//             parse_field_element(s).map_err(|e| {
//                 JsValue::from(&format!(
//                     "Step 6 - Failed to parse public input {}: {:?}",
//                     i, e
//                 ))
//             })
//         })
//         .collect();
//     let public_inputs = public_inputs?;

//     let is_valid = Groth16::<Bn254>::verify_proof(&pvk, &proof, &public_inputs).map_err(|e| {
//         JsValue::from(&format!(
//             "Step 7 - Verify failed (inputs={}): {}",
//             public_inputs.len(),
//             e
//         ))
//     })?;

//     Ok(is_valid)
// }

// // Helper functions
// fn parse_field_element(s: &str) -> Result<Fr, JsValue> {
//     // Handle both decimal and hex strings
//     let s = s.trim();

//     let big_uint = BigUint::from_str(s)
//         .map_err(|e| JsValue::from(&format!("Failed to parse decimal '{}': {}", s, e)))?;
//     Ok(Fr::from(big_uint))
// }

// fn parse_merkle_path(path_data: &[[String; 2]]) -> Result<Path<MERKLE_TREE_LEVEL>, JsValue> {
//     if path_data.len() != MERKLE_TREE_LEVEL {
//         return Err(JsValue::from(&format!(
//             "Invalid Merkle path length: expected {}, got {}",
//             MERKLE_TREE_LEVEL,
//             path_data.len()
//         )));
//     }

//     let mut path = [(Fr::from(0u64), Fr::from(0u64)); MERKLE_TREE_LEVEL];

//     for (i, pair) in path_data.iter().enumerate() {
//         let left = parse_field_element(&pair[0])?;
//         let right = parse_field_element(&pair[1])?;
//         path[i] = (left, right);
//     }

//     Ok(Path { path })
// }
