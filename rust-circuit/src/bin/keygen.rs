use ark_bn254::Bn254;
use ark_groth16::Groth16;
use ark_serialize::CanonicalSerialize;
use rand_chacha::ChaCha20Rng;
use rand_core::SeedableRng;
use std::fs;
use std::path::Path;
use vortex::circuit::TransactionCircuit;
use vortex::poseidon::{poseidon_bn254, PoseidonHash};

pub fn main() -> anyhow::Result<()> {
    println!("Generating Groth16 proving and verifying keys...");

    let hasher = PoseidonHash::new(poseidon_bn254());

    let circuit = TransactionCircuit::empty(hasher);

    // Use deterministic RNG for reproducibility (test mode)
    let mut rng = ChaCha20Rng::from_seed([0u8; 32]);

    println!("Running setup (this may take several minutes)...");
    let pk = Groth16::<Bn254>::generate_random_parameters_with_reduction(circuit, &mut rng)?;
    let vk = pk.vk.clone();

    // Prepare keys directory
    let keys_dir = Path::new("keys");
    if !keys_dir.exists() {
        fs::create_dir_all(keys_dir)?;
    }

    // Serialize verifying key (compressed for smaller size)
    let mut vk_bytes = Vec::new();
    vk.serialize_compressed(&mut vk_bytes)?;

    // Serialize proving key (compressed for smaller size)
    let mut pk_bytes = Vec::new();
    pk.serialize_compressed(&mut pk_bytes)?;

    // Write verifying key (bin + hex)
    fs::write(keys_dir.join("verification_key.bin"), &vk_bytes)?;
    fs::write(
        keys_dir.join("verification_key.hex"),
        hex::encode(&vk_bytes),
    )?;

    // Write proving key (bin + hex)
    fs::write(keys_dir.join("proving_key.bin"), &pk_bytes)?;
    fs::write(keys_dir.join("proving_key.hex"), hex::encode(&pk_bytes))?;

    println!("✅ Keys generated successfully!");
    println!("  Keys written to ./keys/");
    println!("    - proving_key.bin / .hex");
    println!("    - verification_key.bin / .hex");

    Ok(())
}
