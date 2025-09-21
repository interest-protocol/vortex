use crate::{
    merkle_tree::{Path, PathVar},
    poseidon::{PoseidonHash, PoseidonHashVar},
    utils::sha256_hash,
};
use ark_bn254::Fr;
use ark_ff::AdditiveGroup;
use ark_r1cs_std::{
    fields::fp::FpVar,
    prelude::{AllocVar, Boolean, EqGadget},
};
use ark_relations::{
    ns,
    r1cs::{self, ConstraintSynthesizer, ConstraintSystemRef},
};

#[derive(Debug, Clone)]
pub struct Circuit<const L: usize> {
    // Private inputs
    pub secret: Fr,
    pub nullifier: Fr,

    // Public inputs
    pub merkle_root: Fr,
    pub merkle_path: Path<L>,
    pub nullifier_hash: Fr,
    pub recipient: Fr,
    pub relayer: Fr,
    pub relayer_fee: Fr,
    pub vortex: Fr,

    // Constants
    pub hasher: PoseidonHash,
}

impl<const L: usize> Circuit<L> {
    pub fn empty(hash: PoseidonHash) -> Self {
        Self {
            secret: Fr::ZERO,
            nullifier: Fr::ZERO,
            merkle_path: Path::empty(),
            merkle_root: Fr::ZERO,
            nullifier_hash: Fr::ZERO,
            recipient: Fr::ZERO,
            relayer: Fr::ZERO,
            relayer_fee: Fr::ZERO,
            vortex: Fr::ZERO,
            hasher: hash,
        }
    }
}

impl<const L: usize> ConstraintSynthesizer<Fr> for Circuit<L> {
    fn generate_constraints(self, cs: ConstraintSystemRef<Fr>) -> r1cs::Result<()> {
        // Allocate private inputs
        let secret_var = FpVar::new_witness(ns!(cs, "secret"), || Ok(self.secret))?;
        let nullifier_var = FpVar::new_witness(ns!(cs, "nullifier"), || Ok(self.nullifier))?;
        let merkle_path_var =
            PathVar::new_witness(ns!(cs, "merkle_path"), || Ok(self.merkle_path))?;

        // Allocate public inputs
        let merkle_root_var = FpVar::new_input(ns!(cs, "merkle_root"), || Ok(self.merkle_root))?;
        let nullifier_hash_var =
            FpVar::new_input(ns!(cs, "nullifier_hash"), || Ok(self.nullifier_hash))?;
        let _recipient_var = FpVar::new_input(ns!(cs, "recipient"), || Ok(self.recipient))?;
        let _relayer_var = FpVar::new_input(ns!(cs, "relayer"), || Ok(self.relayer))?;
        let _relayer_fee_var = FpVar::new_input(ns!(cs, "relayer_fee"), || Ok(self.relayer_fee))?;
        let _vortex_var = FpVar::new_input(ns!(cs, "vortex"), || Ok(self.vortex))?;

        // Allocate constants
        let hasher_var = PoseidonHashVar::new_constant(ns!(cs, "hasher"), self.hasher)?;

        // CONSTRAINT 1: Verify nullifier hash
        // nullifier_hash = sha256(nullifier)
        let expected_nullifier_hash = FpVar::Constant(sha256_hash(&self.nullifier));

        expected_nullifier_hash.enforce_equal(&nullifier_hash_var)?;

        // Compute commitment = Poseidon(secret, nullifier)
        let commitment = hasher_var.hash(&secret_var, &nullifier_var)?;

        // CONSTRAINT 2: Verify Merkle path
        // merkle_path = Merkle path for commitment in merkle tree
        let is_valid_merkle_path = merkle_path_var
            .check_membership(&merkle_root_var, &commitment, &hasher_var)
            .expect("Invalid Merkle path");

        is_valid_merkle_path.enforce_equal(&Boolean::TRUE)?;

        Ok(())
    }
}
