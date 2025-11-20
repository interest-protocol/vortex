use ark_bn254::Fr;
use vortex::poseidon::{poseidon_bn254, PoseidonHash};

fn main() {
    // Example: hash 3 field elements
    let x = Fr::from(1u64);
    let y = Fr::from(2u64);
    let z = Fr::from(3u64);

    let hasher = PoseidonHash::new(poseidon_bn254());

    let hash1 = hasher.hash1(&x);
    // let hash2 = hasher.hash2(&x, &y);
    // let hash3 = hasher.hash3(&x, &y, &z);

    println!("Poseidon hash1 = {}", hash1);
    // println!("Poseidon hash2 = {}", hash2);
    // println!("Poseidon hash3 = {}", hash3);
}
