use ark_bn254::Fr;
use vortex::poseidon::{poseidon_bn254_t2, poseidon_bn254_t3, poseidon_bn254_t4, PoseidonHash};

fn main() {
    // Example: hash 3 field elements
    let x = Fr::from(1u64);
    let y = Fr::from(2u64);
    let z = Fr::from(3u64);

    let hasher2 = PoseidonHash::new(poseidon_bn254_t2());
    let hasher3 = PoseidonHash::new(poseidon_bn254_t3());
    let hasher4 = PoseidonHash::new(poseidon_bn254_t4());

    let hash1 = hasher2.hash1(&x);
    let hash2 = hasher3.hash2(&x, &y);
    let hash3 = hasher4.hash3(&x, &y, &z);

    println!("Poseidon hash1 = {}", hash1);
    println!("Poseidon hash2 = {}", hash2);
    println!("Poseidon hash3 = {}", hash3);
}
