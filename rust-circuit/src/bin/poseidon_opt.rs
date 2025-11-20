use ark_bn254::Fr;
use vortex::poseidon_opt::{hash1, hash2, hash3};

fn main() {
    // Example: hash 3 field elements
    let x = Fr::from(1u64);
    let y = Fr::from(2u64);
    let z = Fr::from(3u64);

    let hash1 = hash1(&x);
    let hash2 = hash2(&x, &y);
    let hash3 = hash3(&x, &y, &z);

    println!("Poseidon hash1 = {}", hash1);
    println!("Poseidon hash2 = {}", hash2);
    println!("Poseidon hash3 = {}", hash3);
}
