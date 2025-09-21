use ark_bn254::{Bn254, Fr};
use ark_crypto_primitives::snark::SNARK;
use ark_groth16::{Groth16, Proof, VerifyingKey};
use ark_serialize::CanonicalDeserialize;
use serde::Deserialize;
use serde_json::Value;
use std::str::FromStr;

use vortex::utils::{parse_address, sha256_hash};

#[derive(Deserialize, Debug)]
pub struct Params {
    pub nullifier: String,
    pub secret: String,
    pub root: String,
    pub amount: u64,
    pub index: String,
    pub vortex: String,
    pub leafs: Vec<String>,
    pub recipient: String,
    pub relayer: String,
    pub relayer_fee: u64,
    pub nullifier_hash: String,
}

#[derive(Deserialize, Debug)]
pub struct Sui {
    pub package: String,
    pub admin_cap: String,
    pub upgrade_cap: String,
    pub pools: Vec<String>,
}

#[derive(Deserialize, Debug)]
pub struct Config {
    pub prove_params: Params,
    pub sui: Sui,
}

fn main() -> anyhow::Result<()> {
    println!("=== Groth16 Proof Verification Test ===");

    // Load config.json
    let config_str = std::fs::read_to_string("../config.json")?;
    let config: Config = serde_json::from_str(&config_str)?;
    println!("✓ Loaded config.json");

    let vk_bytes = std::fs::read("./keys/vk.bin")?;
    let vk = VerifyingKey::<Bn254>::deserialize_uncompressed(&vk_bytes[..])?;
    println!("✓ Loaded verifying key from proving key");

    // Parse parameters from config.json
    let nullifier_hash =
        Fr::from_str(&config.prove_params.nullifier_hash).expect("Failed to parse nullifier hash");
    let merkle_root = Fr::from_str(&config.prove_params.root).expect("Failed to parse merkle root");
    let relayer_fee = Fr::from(config.prove_params.relayer_fee);

    // Parse addresses from config.json
    let vortex = parse_address(config.prove_params.vortex.clone());
    let recipient = parse_address(config.prove_params.recipient.clone());
    let relayer = parse_address(config.prove_params.relayer.clone());

    println!("✓ Parsed all parameters from config.json");

    // Verify nullifier hash computation
    let nullifier =
        Fr::from_str(&config.prove_params.nullifier).expect("Failed to parse nullifier");
    let expected_nullifier_hash = sha256_hash(&nullifier);
    println!("Expected nullifier hash: {}", expected_nullifier_hash);
    println!("Provided nullifier hash: {}", nullifier_hash);
    assert_eq!(expected_nullifier_hash, nullifier_hash);
    println!("✓ Nullifier hash verification passed");

    // Load proof from file to get merkle path
    let proof_json = std::fs::read_to_string("./keys/proof.json")?;
    let proof_data: Value = serde_json::from_str(&proof_json)?;

    // Use the public inputs directly from the proof.json file
    let public_inputs: Vec<Fr> =
        if let Some(public_inputs_array) = proof_data["public_inputs"].as_array() {
            println!("Using public inputs from proof.json");
            public_inputs_array
                .iter()
                .map(|v| {
                    Fr::from_str(v.as_str().expect("public_input element should be string"))
                        .expect("Failed to parse public_input element")
                })
                .collect()
        } else {
            println!("No public_inputs in proof.json, constructing manually");
            // Parse merkle path from proof
            let merkle_path_elements: Vec<Fr> = proof_data["merkle_path"]
                .as_array()
                .expect("merkle_path should be an array")
                .iter()
                .map(|v| {
                    Fr::from_str(v.as_str().expect("merkle_path element should be string"))
                        .expect("Failed to parse merkle_path element")
                })
                .collect();

            println!("Merkle path has {} elements", merkle_path_elements.len());

            // Prepare public inputs for verification (should match circuit constraint order)
            // Order: 1 (one element), merkle_root, merkle_path (52 elements), nullifier_hash, recipient, relayer, relayer_fee, vortex
            let mut public_inputs = vec![Fr::from(1u64)]; // The "one" element
            public_inputs.push(merkle_root);
            public_inputs.extend(merkle_path_elements);
            public_inputs.extend(vec![
                nullifier_hash,
                recipient,
                relayer,
                relayer_fee,
                vortex,
            ]);
            public_inputs
        };

    println!("=== VERIFICATION DEBUG ===");
    println!("Nullifier: {}", nullifier);
    println!("Nullifier hash: {}", nullifier_hash);
    println!("Merkle root: {}", merkle_root);
    println!("Recipient: {}", recipient);
    println!("Relayer: {}", relayer);
    println!("Relayer fee: {}", relayer_fee);
    println!("Vortex: {}", vortex);
    println!("Public inputs:");
    for (i, input) in public_inputs.iter().enumerate() {
        println!("  {}: {}", i, input);
    }

    println!("Using full proof deserialization");

    let proof = Proof::<Bn254>::deserialize_uncompressed(
        &hex::decode(proof_data["full_proof"].as_str().unwrap())?[..],
    )?;

    println!("✓ Loaded proof from file");

    // Verify proof using direct verify method
    match Groth16::<Bn254>::verify(&vk, &public_inputs, &proof) {
        Ok(is_valid) => {
            if is_valid {
                println!("🎉 PROOF VERIFICATION SUCCESSFUL!");
            } else {
                println!("❌ PROOF VERIFICATION FAILED!");
                return Err(anyhow::anyhow!("Proof verification failed"));
            }
        }
        Err(e) => {
            println!("Verification error: {:?}", e);
            return Err(anyhow::anyhow!("Verification error: {:?}", e));
        }
    }

    Ok(())
}
