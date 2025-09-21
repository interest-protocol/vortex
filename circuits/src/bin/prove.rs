use num_bigint::BigUint;
use serde::Deserialize;
use std::fmt::Debug;
use std::str::FromStr;
use vortex::wasm::{prove, ProveParams};

#[derive(Deserialize, Debug)]
pub struct Params {
    pub nullifier: String,
    pub secret: String,
    pub root: String,
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

pub fn main() -> anyhow::Result<()> {
    let config = std::fs::read_to_string("../config.json").expect("Failed to read config.json");

    let config: Config = serde_json::from_str(&config).expect("Failed to parse config.json");

    let pk_bytes = std::fs::read("./keys/pk.bin").expect("Failed to read pk.bin");

    let mut prove_params = ProveParams {
        secret: config.prove_params.secret,
        nullifier: config.prove_params.nullifier,
        pk_bytes,
        merkle_root: config.prove_params.root,
        nullifier_hash: config.prove_params.nullifier_hash,
        recipient: config.prove_params.recipient,
        relayer: config.prove_params.relayer,
        relayer_fee: config.prove_params.relayer_fee,
        vortex: config.prove_params.vortex,
        deposit_index: config
            .prove_params
            .index
            .parse::<u64>()
            .expect("Failed to parse index"),
        merkle_leafs: vec![],
    };

    // Convert decimal strings to hex strings
    let hex_leafs: Vec<String> = config
        .prove_params
        .leafs
        .iter()
        .map(|s| {
            let bigint = BigUint::from_str(s).expect("Invalid decimal string");
            bigint.to_string()
        })
        .collect();

    prove_params.add_leafs(hex_leafs);

    let proof = prove(prove_params);

    println!("Proof: {}", proof);

    // Save proof to file for verification
    std::fs::write("./keys/proof.json", proof.to_string()).expect("Failed to write proof to file");

    println!("✓ Proof saved to ./keys/proof.json");

    Ok(())
}
