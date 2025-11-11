use ark_bn254::{Bn254, Fr};
use ark_circom::{CircomBuilder, CircomConfig};
use ark_groth16::{Groth16, ProvingKey};
use ark_serialize::CanonicalSerialize;
use rand::{rngs::StdRng, SeedableRng};
use std::fs;
use std::path::Path;

#[tokio::main(flavor = "current_thread")]
async fn main() {
    // 1) Point to your Circom artifacts
    let wasm_path = "../circuits/artifacts/transaction2_js/transaction2.wasm";
    let r1cs_path = "../circuits/artifacts/transaction2.r1cs";

    // 2) Build a Circom config over the field Fr
    let cfg = CircomConfig::<Fr>::new(wasm_path, r1cs_path).expect("failed to load circom config");

    // 3) Create a builder
    let builder = CircomBuilder::new(cfg);

    // Add any inputs here if your circuit requires them for setup
    // builder.push_input("someSignal", 123u64);

    // 4) Build the setup circuit (no witness)
    let circuit = builder.setup();

    // 5) Deterministic RNG (dev only)
    let mut rng = StdRng::seed_from_u64(0);

    // 6) Generate Groth16 parameters
    let pk: ProvingKey<Bn254> =
        Groth16::<Bn254>::generate_random_parameters_with_reduction(circuit, &mut rng)
            .expect("failed to generate parameters");

    // 7) Prepare keys directory
    let keys_dir = Path::new("keys");
    if !keys_dir.exists() {
        fs::create_dir_all(keys_dir).expect("failed to create keys directory");
    }

    // 8) Serialize verifying key
    let mut vk_bytes = Vec::new();
    pk.vk
        .serialize_compressed(&mut vk_bytes)
        .expect("failed to serialize vk");

    // 9) Serialize proving key
    let mut pk_bytes = Vec::new();
    pk.serialize_compressed(&mut pk_bytes)
        .expect("failed to serialize proving key");

    // 10) Write verifying key (bin + hex)
    fs::write(keys_dir.join("verification_key.bin"), &vk_bytes)
        .expect("failed to write verification_key.bin");
    fs::write(
        keys_dir.join("verification_key.hex"),
        hex::encode(&vk_bytes),
    )
    .expect("failed to write verification_key.hex");

    // 11) Write proving key (bin + hex)
    fs::write(keys_dir.join("proving_key.bin"), &pk_bytes)
        .expect("failed to write proving_key.bin");
    fs::write(keys_dir.join("proving_key.hex"), hex::encode(&pk_bytes))
        .expect("failed to write proving_key.hex");

    println!("✅ Keys written to ./keys/");
    println!("  - proving_key.bin / .hex");
    println!("  - verification_key.bin / .hex");
}
