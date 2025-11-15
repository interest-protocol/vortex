use crate::merkle_tree::{Path, PathVar};
use crate::poseidon::{PoseidonHash, PoseidonHashVar};
use ark_bn254::Fr;
use ark_ff::AdditiveGroup;
use ark_r1cs_std::{
    fields::fp::FpVar,
    prelude::{AllocVar, Boolean, EqGadget, FieldVar, ToBitsGadget},
};
use ark_relations::{
    ns,
    r1cs::{self, ConstraintSynthesizer, ConstraintSystemRef},
};

pub const LEVEL: usize = 26;
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

    pub out_public_keys: [Fr; N_OUTS],
    pub out_amounts: [Fr; N_OUTS],
    pub out_blindings: [Fr; N_OUTS],
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

            out_public_keys: [Fr::ZERO; N_OUTS],
            out_amounts: [Fr::ZERO; N_OUTS],
            out_blindings: [Fr::ZERO; N_OUTS],
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

        let mut sum_outs = FpVar::<Fr>::zero();

        let out_public_key = [
            FpVar::new_input(ns!(cs, "out_public_key_0"), || Ok(self.out_public_keys[0]))?,
            FpVar::new_input(ns!(cs, "out_public_key_1"), || Ok(self.out_public_keys[1]))?,
        ];

        let out_amounts = [
            FpVar::new_input(ns!(cs, "out_amount_0"), || Ok(self.out_amounts[0]))?,
            FpVar::new_input(ns!(cs, "out_amount_1"), || Ok(self.out_amounts[1]))?,
        ];

        let out_blindings = [
            FpVar::new_input(ns!(cs, "out_blinding_0"), || Ok(self.out_blindings[0]))?,
            FpVar::new_input(ns!(cs, "out_blinding_1"), || Ok(self.out_blindings[1]))?,
        ];

        for i in 0..N_OUTS {
            let out_amount = out_amounts[i].clone();

            let expected_commitment =
                hasher.hash3(&out_amount, &out_public_key[i], &out_blindings[i])?;

            expected_commitment.enforce_equal(&output_commitment[i])?;

            // Check that out_amount fits into 248 bits
            // BN254 field has 254 bits, so we check bits 248..254 (indices 248-253) are zero
            let bits = out_amount.to_bits_le()?;
            // Ensure we have enough bits (should be 254 for BN254)
            if bits.len() < 254 {
                return Err(r1cs::SynthesisError::Unsatisfiable);
            }
            // Check that the top 6 bits (248-253) are all zero
            for bit in &bits[248..254] {
                bit.enforce_equal(&Boolean::constant(false))?;
            }

            sum_outs += &out_amount;
        }

        // Check that there are no same nullifiers among all inputs
        // Equivalent to Circom: sameNullifiers[index].out === 0 (i.e., values are NOT equal)
        // Only check pairs where j > i to avoid duplicate checks (nIns * (nIns - 1) / 2 pairs)
        for i in 0..(N_INS - 1) {
            for j in (i + 1)..N_INS {
                let are_not_equal = input_nullifiers[i].is_neq(&input_nullifiers[j])?;
                // Enforce that they are NOT equal (equivalent to IsEqual().out === 0 in Circom)
                are_not_equal.enforce_equal(&Boolean::constant(true))?;
            }
        }

        (sum_ins + public_amount).enforce_equal(&sum_outs)?;

        // optional safety constraint to make sure extDataHash cannot be changed
        // This creates a constraint that ext_data_hash^2 = ext_data_square
        let _ext_data_square = &ext_data_hash * &ext_data_hash;

        Ok(())
    }
}
