use crate::merkle_tree::{Path, PathVar};
use crate::poseidon::{PoseidonHash, PoseidonHashVar};
use ark_bn254::Fr;
use ark_ff::AdditiveGroup;
use ark_r1cs_std::{
    fields::fp::FpVar,
    prelude::{AllocVar, Boolean, EqGadget, FieldVar},
};
use ark_relations::{
    ns,
    r1cs::{self, ConstraintSynthesizer, ConstraintSystemRef},
};

pub const LEVEL: usize = 26;

pub const ZERO_VALUE: &str =
    "18688842432741139442778047327644092677418528270738216181718229581494125774932";

pub const N_INS: usize = 2;
pub const N_OUTS: usize = 2;

#[derive(Debug, Clone)]
pub struct TransactionCircuit {
    // Constants
    pub hasher: PoseidonHash,

    // Public inputs
    pub root: Fr,
    pub public_amount: Fr,
    pub ext_data_hash: Fr,
    pub input_nullifiers: [Fr; N_INS],
    pub output_commitment: [Fr; N_OUTS],

    // Private inputs
    pub in_private_keys: [Fr; N_INS],
    pub in_amounts: [Fr; N_INS],
    pub in_blindings: [Fr; N_INS],
    pub in_path_indices: [Fr; N_INS],
    pub merkle_paths: [Path<LEVEL>; N_INS],
}

impl TransactionCircuit {
    pub fn empty(hash: PoseidonHash) -> Self {
        Self {
            hasher: hash,

            root: Fr::ZERO,
            public_amount: Fr::ZERO,
            ext_data_hash: Fr::ZERO,
            input_nullifiers: [Fr::ZERO; N_INS],
            output_commitment: [Fr::ZERO; N_OUTS],

            in_private_keys: [Fr::ZERO; N_INS],
            in_amounts: [Fr::ZERO; N_INS],
            in_blindings: [Fr::ZERO; N_INS],
            in_path_indices: [Fr::ZERO; N_INS],
            merkle_paths: [Path::empty(); N_INS],
        }
    }
}

impl ConstraintSynthesizer<Fr> for TransactionCircuit {
    fn generate_constraints(self, cs: ConstraintSystemRef<Fr>) -> r1cs::Result<()> {
        // Allocate constants
        let hasher = PoseidonHashVar::new_constant(ns!(cs, "hasher"), self.hasher)?;

        // Allocate public inputs
        let root = FpVar::new_input(ns!(cs, "root"), || Ok(self.root))?;
        let public_amount = FpVar::new_input(ns!(cs, "public_amount"), || Ok(self.public_amount))?;
        let ext_data_hash = FpVar::new_input(ns!(cs, "ext_data_hash"), || Ok(self.ext_data_hash))?;

        // Allocate array elements individually
        let input_nullifiers = [
            FpVar::new_input(
                ns!(cs, "input_nullifier_0"),
                || Ok(self.input_nullifiers[0]),
            )?,
            FpVar::new_input(
                ns!(cs, "input_nullifier_1"),
                || Ok(self.input_nullifiers[1]),
            )?,
        ];

        let output_commitment = [
            FpVar::new_input(ns!(cs, "output_commitment_0"), || {
                Ok(self.output_commitment[0])
            })?,
            FpVar::new_input(ns!(cs, "output_commitment_1"), || {
                Ok(self.output_commitment[1])
            })?,
        ];

        let in_private_key = [
            FpVar::new_input(ns!(cs, "in_private_key_0"), || Ok(self.in_private_keys[0]))?,
            FpVar::new_input(ns!(cs, "in_private_key_1"), || Ok(self.in_private_keys[1]))?,
        ];

        let in_amounts = [
            FpVar::new_input(ns!(cs, "in_amount_0"), || Ok(self.in_amounts[0]))?,
            FpVar::new_input(ns!(cs, "in_amount_1"), || Ok(self.in_amounts[1]))?,
        ];

        let in_blindings = [
            FpVar::new_input(ns!(cs, "in_blinding_0"), || Ok(self.in_blindings[0]))?,
            FpVar::new_input(ns!(cs, "in_blinding_1"), || Ok(self.in_blindings[1]))?,
        ];

        let in_path_indices = [
            FpVar::new_input(ns!(cs, "in_path_index_0"), || Ok(self.in_path_indices[0]))?,
            FpVar::new_input(ns!(cs, "in_path_index_1"), || Ok(self.in_path_indices[1]))?,
        ];

        let merkle_paths = [
            PathVar::new_input(ns!(cs, "merkle_path_0"), || Ok(self.merkle_paths[0]))?,
            PathVar::new_input(ns!(cs, "merkle_path_1"), || Ok(self.merkle_paths[1]))?,
        ];

        let mut sum_ins = FpVar::<Fr>::zero();

        for i in 0..N_INS {
            let in_private_key = in_private_key[i].clone();
            let in_amount = in_amounts[i].clone();
            let in_blinding = in_blindings[i].clone();
            let in_path_index = in_path_indices[i].clone();

            let public_key = hasher.hash1(&in_private_key)?;

            let commitment = hasher.hash3(&in_amount, &public_key, &in_blinding)?;

            let signature = hasher.hash3(&in_private_key, &commitment, &in_path_index)?;

            let nullifier = hasher.hash3(&commitment, &in_path_index, &signature)?;

            nullifier.enforce_equal(&input_nullifiers[i])?;

            let zero = FpVar::<Fr>::zero();
            let amount_is_zero = in_amount.is_eq(&zero)?;

            let merkle_path = merkle_paths[i].clone();
            let merkle_path_membership =
                merkle_path.check_membership(&root, &commitment, &hasher)?;

            // Conditional enforcement using boolean logic:
            // "if amount is not zero => membership must be true"
            // This is equivalent to: "amount_is_zero OR membership = true"
            // When amount is zero: constraint is satisfied (no check needed)
            // When amount is not zero: membership must be true (enforced)
            let condition = &amount_is_zero | &merkle_path_membership;
            condition.enforce_equal(&Boolean::constant(true))?;

            sum_ins += &in_amount;
        }

        Ok(())
    }
}
