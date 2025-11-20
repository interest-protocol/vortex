use ark_bn254::Fr;
use vortex::poseidon::{poseidon_hash1, poseidon_hash2, poseidon_hash3};

fn main() {
    // Example: hash 3 field elements
    let x = Fr::from(1u64);
    let y = Fr::from(2u64);
    let z = Fr::from(3u64);

    let hash1 = poseidon_hash1(x).expect("Poseidon hash failed");
    let hash2 = poseidon_hash2(x, y).expect("Poseidon hash failed");
    let hash3 = poseidon_hash3(x, y, z).expect("Poseidon hash failed");

    println!("Poseidon hash1 = {}", hash1);
    println!("Poseidon hash2 = {}", hash2);
    println!("Poseidon hash3 = {}", hash3);
}
