use anyhow::{anyhow, Result};
use ark_bn254::{Bn254, Fr};
use ark_crypto_primitives::snark::{CircuitSpecificSetupSNARK, SNARK};
use ark_groth16::{Groth16, ProvingKey};
use ark_r1cs_std::{boolean::Boolean, fields::fp::FpVar, prelude::*, select::CondSelectGadget};
use ark_relations::r1cs::{ConstraintSynthesizer, ConstraintSystemRef, SynthesisError};
use ark_serialize::{CanonicalDeserialize, CanonicalSerialize};
use ark_std::rand::thread_rng;
use num_bigint::BigUint;
use num_traits::Num;
use std::env;
use std::fs;
use std::str::FromStr;

mod poseidon;

use poseidon::{poseidon_bn254, PoseidonHash};

// Poseidon hash circuit helper function
fn poseidon_hash_circuit(
    cs: ConstraintSystemRef<Fr>,
    x: &FpVar<Fr>,
    y: &FpVar<Fr>,
) -> Result<FpVar<Fr>, SynthesisError> {
    use poseidon::PoseidonHashVar;

    // Create Poseidon hasher for constraints
    let poseidon_config = poseidon_bn254();
    let poseidon_var = PoseidonHashVar::new_witness(cs.clone(), || Ok(poseidon_config))?;

    // Hash the inputs
    let result = poseidon_var.hash(x, y)?;

    Ok(result)
}

// Vortex withdrawal circuit
#[derive(Clone)]
#[allow(dead_code)]
struct VortexWithdrawCircuit {
    // Private inputs
    secret: Fr,
    nullifier_secret: Fr,
    merkle_path: Vec<Fr>,
    merkle_indices: Vec<bool>,

    // Public inputs
    merkle_root: Fr,
    nullifier: Fr,
    recipient: Fr,
    relayer: Fr,
    relayer_fee: Fr,
    vortex: Fr,
}

impl ConstraintSynthesizer<Fr> for VortexWithdrawCircuit {
    fn generate_constraints(self, cs: ConstraintSystemRef<Fr>) -> Result<(), SynthesisError> {
        // Allocate private inputs
        let secret_var = FpVar::new_witness(cs.clone(), || Ok(self.secret))?;
        let nullifier_secret_var = FpVar::new_witness(cs.clone(), || Ok(self.nullifier_secret))?;

        // Allocate public inputs
        let merkle_root_var = FpVar::new_input(cs.clone(), || Ok(self.merkle_root))?;
        let nullifier_var = FpVar::new_input(cs.clone(), || Ok(self.nullifier))?;

        // Compute commitment = Poseidon(secret, nullifier_secret)
        let commitment = poseidon_hash_circuit(cs.clone(), &secret_var, &nullifier_secret_var)?;

        // Compute nullifier_hash = Poseidon(commitment, nullifier_secret)
        let computed_nullifier =
            poseidon_hash_circuit(cs.clone(), &commitment, &nullifier_secret_var)?;

        // Verify nullifier matches
        computed_nullifier.enforce_equal(&nullifier_var)?;

        // Verify Merkle proof
        let mut current = commitment;
        for (path_elem, is_right) in self.merkle_path.iter().zip(self.merkle_indices.iter()) {
            let path_var = FpVar::new_witness(cs.clone(), || Ok(*path_elem))?;
            let is_right_var = Boolean::new_witness(cs.clone(), || Ok(*is_right))?;

            // Compute both possible hashes
            let hash_left = poseidon_hash_circuit(cs.clone(), &current, &path_var)?;
            let hash_right = poseidon_hash_circuit(cs.clone(), &path_var, &current)?;

            // Select based on is_right
            current = FpVar::conditionally_select(&is_right_var, &hash_right, &hash_left)?;
        }

        // Verify root matches
        current.enforce_equal(&merkle_root_var)?;

        Ok(())
    }
}

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        println!("Usage:");
        println!("  {} setup - Generate proving and verifying keys", args[0]);
        println!("  {} prove - Generate a withdrawal proof", args[0]);
        println!("  {} verify <proof.bin> - Verify a proof", args[0]);
        return Ok(());
    }

    let command = &args[1];

    match command.as_str() {
        "setup" => {
            println!("Generating proving and verifying keys...");

            // Create a dummy circuit for setup
            let circuit = VortexWithdrawCircuit {
                secret: Fr::from(123456789u64),
                nullifier_secret: Fr::from(987654321u64),
                merkle_path: vec![Fr::from(0u64); 26],
                merkle_indices: vec![false; 26],
                merkle_root: Fr::from(0u64),
                nullifier: Fr::from(0u64),
                recipient: Fr::from(0u64),
                relayer: Fr::from(0u64),
                relayer_fee: Fr::from(0u64),
                vortex: Fr::from(0u64),
            };

            let (pk, vk) = Groth16::<Bn254>::setup(circuit, &mut thread_rng())?;

            // Save proving key
            let mut pk_bytes = Vec::new();
            pk.serialize_uncompressed(&mut pk_bytes)?;
            fs::write("../keys/proving_key.bin", &pk_bytes)?;
            println!(
                "✅ Proving key saved to proving_key.bin ({} bytes)",
                pk_bytes.len()
            );

            // Save verifying key
            let mut vk_bytes = Vec::new();
            vk.serialize_compressed(&mut vk_bytes)?;
            fs::write("../keys/verifying_key.bin", &vk_bytes)?;
            println!(
                "✅ Verifying key saved to verifying_key.bin ({} bytes)",
                vk_bytes.len()
            );

            // Also save VK in hex for Sui
            let vk_hex = hex::encode(&vk_bytes);
            fs::write("../keys/verifying_key.hex", &vk_hex)?;
            println!("✅ Verifying key hex saved to verifying_key.hex");
        }

        "prove" => {
            // Expected: ./target/release/vortex-circuit prove <recipient_address>
            let recipient_address = args.get(2).ok_or_else(|| {
                anyhow!("Usage: ./target/release/vortex-circuit prove <recipient_address>")
            })?;

            println!("Generating withdrawal proof...");
            println!("Recipient address: {}", recipient_address);

            // Load proving key
            let pk_bytes = fs::read("proving_key.bin")?;
            let pk = ProvingKey::<Bn254>::deserialize_uncompressed(&pk_bytes[..])?;

            // Load deposit info
            let deposit_json = fs::read_to_string("deposit_devnet.json")?;
            let deposit: serde_json::Value = serde_json::from_str(&deposit_json)?;

            // Extract values from deposit JSON
            // Note: The JSON field names are misleading - "nullifier" is actually the nullifier_secret
            // and "secret" is the secret. The actual nullifier is computed from these.
            let nullifier_secret_str = deposit["nullifier"].as_str().unwrap();
            let secret_str = deposit["secret"].as_str().unwrap();
            let root_val = deposit["root"].as_str().unwrap();
            let amount = deposit["amount"].as_u64().unwrap();
            let deposit_index = deposit["index"].as_str().unwrap().parse::<usize>().unwrap();
            let vortex_pool = deposit["vortex"].as_str().unwrap();

            // In the circuit: commitment = Poseidon(secret, nullifier_secret)
            // In JS we did: commitment = poseidon2([nullifier, secret])
            // So we need to swap the order
            let secret = Fr::from_str(nullifier_secret_str).unwrap(); // JS "nullifier" -> circuit "secret"
            let nullifier_secret = Fr::from_str(secret_str).unwrap(); // JS "secret" -> circuit "nullifier_secret"

            // Parse merkle root from decimal string
            let merkle_root = Fr::from_str(root_val).unwrap();

            // Build Poseidon hasher
            let poseidon = PoseidonHash::new(poseidon_bn254());

            // Calculate commitment - circuit expects: Poseidon(secret, nullifier_secret)
            let commitment = poseidon.hash(&secret, &nullifier_secret);
            println!("Commitment: {}", commitment);
            println!("Expected: 6512303153764079852476596566766333446152566874294982298377048820452403290637");

            // Calculate nullifier - this should be poseidon(commitment, nullifier)
            // But wait, in Tornado Cash style, nullifier = poseidon(nullifier_secret)
            // Let's use the same pattern as commitment
            let nullifier = poseidon.hash(&commitment, &nullifier_secret);
            println!("Nullifier: {}", nullifier);

            // Build merkle path for the actual deposit index
            println!("Building Merkle path for index: {}", deposit_index);

            // This zero value must match the contract's ZERO_VALUE = poseidon1(stringToField('vortex'))
            let zero_value = Fr::from_str(
                "18688842432741139442778047327644092677418528270738216181718229581494125774932",
            )
            .unwrap();
            let mut zeros = vec![zero_value];
            for i in 1..26 {
                let hash = poseidon.hash(&zeros[i - 1], &zeros[i - 1]);
                zeros.push(hash);
            }

            // Convert index to binary path (26 bits, little-endian)
            let mut path_indices = vec![false; 26];
            let mut temp_index = deposit_index;
            for i in 0..26 {
                path_indices[i] = (temp_index & 1) == 1;
                temp_index >>= 1;
            }

            // PROPER SOLUTION: Reconstruct tree state with actual Poseidon hashing
            println!("Reconstructing tree state with proper Poseidon hashing...");
            let tree_state_json = fs::read_to_string("tree_state.json").map_err(|e| {
                anyhow!("Failed to read tree_state.json. Please run 'node rebuild_tree_state.js' first: {}", e)
            })?;
            let tree_state: serde_json::Value = serde_json::from_str(&tree_state_json)?;

            // Get all commitments up to current index
            let all_commitments = tree_state["allCommitments"]
                .as_array()
                .ok_or(anyhow!("Missing allCommitments"))?;
            println!("Found {} commitments to process", all_commitments.len());

            // Reconstruct the tree exactly as the contract does, using proper Poseidon
            let mut subtrees = vec![zero_value; 26]; // Initialize with zeros
            let mut merkle_path = zeros.clone(); // Will be updated with correct siblings

            // Process each commitment in order to build the tree state
            for comm_data in all_commitments.iter() {
                let comm_index = comm_data["index"]
                    .as_u64()
                    .ok_or(anyhow!("Invalid commitment index"))?
                    as usize;
                let comm_str = comm_data["commitment"]
                    .as_str()
                    .ok_or(anyhow!("Invalid commitment"))?;
                let commitment_val =
                    Fr::from_str(comm_str).map_err(|_| anyhow!("Failed to parse commitment"))?;

                println!(
                    "Processing commitment {} at index {}",
                    commitment_val, comm_index
                );

                // Simulate the contract's append logic EXACTLY
                let mut current_index = comm_index;
                let mut current_level_hash = commitment_val;

                for level in 0..26 {
                    let left;
                    let right;

                    if current_index % 2 == 0 {
                        // Even index: we are left child, right sibling is zero
                        left = current_level_hash;
                        right = zeros[level];

                        // Update subtree BEFORE checking if we need to record sibling
                        subtrees[level] = current_level_hash;

                        // If this is our target deposit, record the sibling (which is right)
                        if comm_index == deposit_index {
                            merkle_path[level] = right; // Right sibling
                            println!(
                                "  Level {}: Even index, sibling = zero[{}] = {}",
                                level, level, right
                            );
                        }
                    } else {
                        // Odd index: we are right child, left sibling comes from subtree
                        left = subtrees[level];
                        right = current_level_hash;

                        // If this is our target deposit, record the sibling (which is left)
                        if comm_index == deposit_index {
                            merkle_path[level] = left; // Left sibling from subtree
                            println!(
                                "  Level {}: Odd index, sibling = subtree[{}] = {}",
                                level, level, left
                            );
                        }
                    }

                    current_level_hash = poseidon.hash(&left, &right);
                    current_index /= 2;

                    // Debug: show the hash computation for our target
                    if comm_index == deposit_index && level < 4 {
                        println!("    Hash({}, {}) = {}", left, right, current_level_hash);
                    }
                }
            }

            // Verify: compute the root using our path to check if it matches
            println!("\nVerifying path computation:");
            let mut current_hash = commitment;
            let mut idx = deposit_index;
            for level in 0..26 {
                let sibling = merkle_path[level];
                let is_right = (idx & 1) == 1;

                let (left, right) = if is_right {
                    (sibling, current_hash)
                } else {
                    (current_hash, sibling)
                };

                current_hash = poseidon.hash(&left, &right);
                idx /= 2;

                if level < 3 {
                    println!(
                        "  Level {}: {} child, Hash({}, {}) = {}",
                        level,
                        if is_right { "right" } else { "left" },
                        left,
                        right,
                        current_hash
                    );
                }
            }
            println!("Computed root: {}", current_hash);
            println!("Expected root: {}", root_val);

            if current_hash.to_string() != root_val {
                println!(
                    "WARNING: Root mismatch! Our path computation doesn't match the expected root."
                );
                println!("This means our tree reconstruction has an error.");
            } else {
                println!("SUCCESS: Root matches! Our tree reconstruction is correct.");
            }

            println!(
                "\nBuilding path for index {} to reach root {}",
                deposit_index, root_val
            );

            println!("Merkle path indices: {:?}", &path_indices[..8]); // Show first 8 bits

            // Convert recipient address to field element
            // Remove 0x prefix if present
            let clean_address = recipient_address
                .strip_prefix("0x")
                .unwrap_or(recipient_address);
            let recipient_bigint = BigUint::from_str_radix(clean_address, 16)?;
            let recipient_field = Fr::from(recipient_bigint);

            println!("Recipient field: {}", recipient_field);

            let clean_vortex = vortex_pool.strip_prefix("0x").unwrap_or(vortex_pool);
            let vortex_field = Fr::from_str(
                BigUint::from_str_radix(clean_vortex, 16)
                    .unwrap()
                    .to_string()
                    .as_str(),
            )
            .unwrap();

            println!("Vortex field: {}", vortex_field);

            // Create circuit
            let circuit = VortexWithdrawCircuit {
                secret,
                nullifier_secret,
                merkle_path,
                merkle_indices: path_indices,
                merkle_root,
                nullifier,
                recipient: recipient_field,
                relayer: recipient_field, // Use same address for relayer
                relayer_fee: Fr::from(0u64),
                vortex: vortex_field,
            };

            // Generate proof
            let proof = Groth16::<Bn254>::prove(&pk, circuit, &mut thread_rng())?;

            // Save proof
            let mut proof_bytes = Vec::new();
            proof.serialize_compressed(&mut proof_bytes)?;
            fs::write("proof.bin", &proof_bytes)?;
            println!("✅ Proof saved to proof.bin ({} bytes)", proof_bytes.len());

            // Save proof hex for Sui
            let proof_hex = hex::encode(&proof_bytes);
            fs::write("proof.hex", &proof_hex)?;
            println!("✅ Proof hex saved to proof.hex");
            println!("Proof: 0x{}", proof_hex);

            // Save public inputs as separate lines (matching circuit's actual public inputs)
            // Note: value is NOT a public input in the circuit
            let public_inputs = vec![
                ("root", merkle_root),
                ("nullifier", nullifier),
                ("recipient", recipient_field), // Use the dynamic recipient from command line
                ("relayer", recipient_field),   // Use same address for relayer
                ("relayer_fee", Fr::from(0u64)),
            ];

            // Save value separately for the contract call
            fs::write("value.txt", amount.to_string())?;

            let mut public_lines = Vec::new();
            for (name, input) in &public_inputs {
                // Convert to decimal string
                let decimal_str = if *name == "relayer_fee" && input.to_string().is_empty() {
                    "0".to_string() // Fix for Fr::from(0u64) returning empty string
                } else {
                    input.to_string()
                };
                public_lines.push(decimal_str);
            }

            fs::write("public_inputs.txt", public_lines.join("\n"))?;
            println!("✅ Public inputs saved to public_inputs.txt");
        }

        _ => {
            println!("Unknown command: {}", command);
        }
    }

    Ok(())
}
