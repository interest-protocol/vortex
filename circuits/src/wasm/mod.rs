use crate::utils::parse_address;
use ark_bn254::{Bn254, Fr};
use ark_crypto_primitives::snark::SNARK;
use ark_groth16::{Groth16, ProvingKey};
use ark_relations::r1cs::{ConstraintSynthesizer, ConstraintSystem};
use ark_serialize::{CanonicalDeserialize, CanonicalSerialize};
use ark_std::rand::thread_rng;
use num_bigint::BigUint;
use num_traits::ToPrimitive;
use serde_json::{json, Value};

use std::str::FromStr;

use crate::{
    circuit::Circuit,
    merkle_tree::SparseMerkleTree,
    poseidon::{poseidon_bn254, PoseidonHash},
    LEVEL, ZERO_VALUE,
};

#[derive(Debug)]
pub struct ProveParams {
    pub secret: String,
    pub nullifier_hash: String,
    pub pk_bytes: Vec<u8>,
    pub merkle_root: String,
    pub nullifier: String,
    pub recipient: String,
    pub relayer: String,
    pub relayer_fee: u64,
    pub vortex: String,
    pub deposit_index: u64,
    pub merkle_leafs: Vec<Fr>,
}

impl ProveParams {
    pub fn add_leafs(&mut self, leafs: Vec<String>) {
        self.merkle_leafs.extend(leafs.iter().map(|s| {
            // Convert hex string back to decimal, then to field element
            let bigint = BigUint::from_str(s).expect("Invalid hex string");
            Fr::from(bigint)
        }));
    }
}

pub fn prove(params: ProveParams) -> Value {
    let pk = ProvingKey::<Bn254>::deserialize_uncompressed(&params.pk_bytes[..])
        .expect("Failed to deserialize proving key");

    let secret = Fr::from_str(&params.secret).expect("Failed to parse secret");
    let nullifier_hash =
        Fr::from_str(&params.nullifier_hash).expect("Failed to parse nullifier hash");
    let merkle_root = Fr::from_str(&params.merkle_root).expect("Failed to parse merkle root");
    let nullifier = Fr::from_str(&params.nullifier).expect("Failed to parse nullifier");
    let relayer_fee = Fr::from(params.relayer_fee);
    let deposit_index = params.deposit_index;

    // Addresses to field elements
    let vortex = parse_address(params.vortex);
    let recipient = parse_address(params.recipient);
    let relayer = parse_address(params.relayer);

    let poseidon = PoseidonHash::new(poseidon_bn254());

    let merkle_tree = SparseMerkleTree::<LEVEL>::new_sequential(
        &params.merkle_leafs,
        &poseidon,
        &Fr::from_str(ZERO_VALUE).unwrap(),
    )
    .expect("Invalid merkle tree construction");

    assert!(merkle_tree.root().eq(&merkle_root));

    let merkle_path = deposit_index
        .to_usize()
        .map(|i| merkle_tree.generate_membership_proof(i as u64))
        .unwrap_or(merkle_tree.generate_membership_proof(0));

    let circuit = Circuit {
        secret,
        nullifier_hash,
        merkle_root,
        merkle_path,
        nullifier,
        recipient,
        relayer,
        relayer_fee,
        vortex,
        hasher: poseidon.clone(),
    };

    let cs = ConstraintSystem::<Fr>::new_ref();
    circuit
        .clone()
        .generate_constraints(cs.clone())
        .expect("Failed to generate constraints");

    if !cs.is_satisfied().expect("Failed to check constraints") {
        panic!("Constraints are not satisfied");
    }

    // Generate proof
    let proof = Groth16::<Bn254>::prove(&pk, circuit.clone(), &mut thread_rng())
        .expect("Failed to generate proof");

    // Skip verification during proving for now - we'll verify separately

    // Serialize the entire proof for verification
    let mut proof_bytes = vec![];
    proof
        .serialize_uncompressed(&mut proof_bytes)
        .expect("Failed to serialize proof");

    // Serialize merkle path for verification
    let mut merkle_path_elements = vec![];
    for (left, right) in merkle_path.path.iter() {
        merkle_path_elements.push(left.to_string());
        merkle_path_elements.push(right.to_string());
    }

    json!({
        "full_proof": hex::encode(proof_bytes),
        "merkle_path": merkle_path_elements,

    })
}
