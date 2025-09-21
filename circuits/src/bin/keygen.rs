use ark_bn254::Bn254;
use ark_crypto_primitives::snark::CircuitSpecificSetupSNARK;
use ark_groth16::Groth16;
use ark_serialize::CanonicalSerialize;

use rand::thread_rng;
use std::fs::File;
use std::io::Write;
use vortex::{
    circuit::Circuit,
    poseidon::{poseidon_bn254, PoseidonHash},
    LEVEL,
};

pub fn main() -> anyhow::Result<()> {
    let poseidon = PoseidonHash::new(poseidon_bn254());
    let circuit = Circuit::<LEVEL>::empty(poseidon);

    let (pk, vk) = Groth16::<Bn254>::setup(circuit, &mut thread_rng())?;

    // Serialize PK uncompressed
    let mut pk_bytes = vec![];
    pk.serialize_uncompressed(&mut pk_bytes)?;

    // Serialize VK compressed
    let mut vk_bytes = vec![];
    vk.serialize_uncompressed(&mut vk_bytes)?;

    // Save PK to file
    let mut pk_file = File::create("./keys/pk.bin")?;
    pk_file.write_all(&pk_bytes)?;

    // Save VK as uncompressed binary bytes
    let mut vk_file = File::create("./keys/vk.bin")?;
    vk_file.write_all(&vk_bytes)?;

    // Also save VK as compressed for reference

    let vk_hex = hex::encode(&vk_bytes);
    let mut vk_hex_file = File::create("./keys/vk.hex.bin")?;
    vk_hex_file.write_all(vk_hex.as_bytes())?;

    println!("Keys saved to ./keys/pk.bin, ./keys/vk.bin and ./keys/vk.hex.bin");

    Ok(())
}
