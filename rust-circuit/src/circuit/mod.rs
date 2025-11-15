use crate::{
    constants::{LEVEL, N_INS, N_OUTS},
    merkle_tree::{Path, PathVar},
    poseidon::{PoseidonHash, PoseidonHashVar},
};
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
use std::ops::Not;

/// Transaction circuit for privacy-preserving value transfers on Sui.
///
/// This circuit implements a 2-input, 2-output transaction model where:
/// - Users can spend up to 2 input UTXOs (zero amounts allowed)
/// - Create up to 2 output UTXOs (zero amounts allowed)
/// - Add/remove value from the pool via `public_amount`
///
/// # Privacy Guarantees
///
/// - Input amounts, recipients, and senders are hidden
/// - Only nullifiers and output commitments are public
/// - Links between inputs and outputs are obfuscated
///
/// # Security Properties
///
/// 1. **No double-spending**: Each nullifier can only be used once
/// 2. **Amount conservation**: Σinputs + public_amount = Σoutputs  
/// 3. **Valid proofs**: All non-zero inputs have valid Merkle proofs
/// 4. **No overflow**: All amounts fit in 248 bits
/// 5. **Unique nullifiers**: No duplicate nullifiers in same transaction
#[derive(Debug, Clone)]
pub struct TransactionCircuit {
    // Constants
    pub hasher: PoseidonHash,

    // Public inputs (must match order expected by Move contract verification)
    pub root: Fr,
    pub public_amount: Fr,
    pub ext_data_hash: Fr,
    pub input_nullifiers: [Fr; N_INS],
    pub output_commitment: [Fr; N_OUTS],

    // Private inputs - Input UTXOs
    pub in_private_keys: [Fr; N_INS],
    pub in_amounts: [Fr; N_INS],
    pub in_blindings: [Fr; N_INS],
    pub in_path_indices: [Fr; N_INS],
    pub merkle_paths: [Path<LEVEL>; N_INS],

    // Private inputs - Output UTXOs
    pub out_public_keys: [Fr; N_OUTS],
    pub out_amounts: [Fr; N_OUTS],
    pub out_blindings: [Fr; N_OUTS],
}

impl TransactionCircuit {
    /// Creates an empty circuit with all values set to zero.
    /// Used for setup phase and testing.
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
        // ============================================
        // ALLOCATE CONSTANTS
        // ============================================
        let hasher = PoseidonHashVar::new_constant(ns!(cs, "hasher"), self.hasher)?;

        // ============================================
        // ALLOCATE PUBLIC INPUTS
        // Order must match Move contract's verification expectations
        // ============================================
        let root = FpVar::new_input(ns!(cs, "root"), || Ok(self.root))?;
        let public_amount = FpVar::new_input(ns!(cs, "public_amount"), || Ok(self.public_amount))?;
        let ext_data_hash = FpVar::new_input(ns!(cs, "ext_data_hash"), || Ok(self.ext_data_hash))?;

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

        // ============================================
        // ALLOCATE PRIVATE WITNESS INPUTS
        // ============================================
        let in_private_key = [
            FpVar::new_witness(ns!(cs, "in_private_key_0"), || Ok(self.in_private_keys[0]))?,
            FpVar::new_witness(ns!(cs, "in_private_key_1"), || Ok(self.in_private_keys[1]))?,
        ];

        let in_amounts = [
            FpVar::new_witness(ns!(cs, "in_amount_0"), || Ok(self.in_amounts[0]))?,
            FpVar::new_witness(ns!(cs, "in_amount_1"), || Ok(self.in_amounts[1]))?,
        ];

        let in_blindings = [
            FpVar::new_witness(ns!(cs, "in_blinding_0"), || Ok(self.in_blindings[0]))?,
            FpVar::new_witness(ns!(cs, "in_blinding_1"), || Ok(self.in_blindings[1]))?,
        ];

        let in_path_indices = [
            FpVar::new_witness(ns!(cs, "in_path_index_0"), || Ok(self.in_path_indices[0]))?,
            FpVar::new_witness(ns!(cs, "in_path_index_1"), || Ok(self.in_path_indices[1]))?,
        ];

        let merkle_paths = [
            PathVar::new_witness(ns!(cs, "merkle_path_0"), || Ok(self.merkle_paths[0]))?,
            PathVar::new_witness(ns!(cs, "merkle_path_1"), || Ok(self.merkle_paths[1]))?,
        ];

        // ============================================
        // VERIFY INPUT UTXOs
        // ============================================
        let mut sum_ins = FpVar::<Fr>::zero();
        let zero = FpVar::<Fr>::zero();

        for i in 0..N_INS {
            let in_private_key = in_private_key[i].clone();
            let in_amount = in_amounts[i].clone();
            let in_blinding = in_blindings[i].clone();
            let in_path_index = in_path_indices[i].clone();

            // Derive public key from private key: pubkey = Poseidon1(privkey)
            let public_key = hasher.hash1(&in_private_key)?;

            // Calculate commitment: commitment = Poseidon3(amount, pubkey, blinding)
            // Note: We intentionally removed mint_address from commitments
            let commitment = hasher.hash3(&in_amount, &public_key, &in_blinding)?;

            // Calculate signature: sig = Poseidon3(privkey, commitment, path_index)
            let signature = hasher.hash3(&in_private_key, &commitment, &in_path_index)?;

            // Calculate nullifier: nullifier = Poseidon3(commitment, path_index, signature)
            let nullifier = hasher.hash3(&commitment, &in_path_index, &signature)?;

            // Enforce computed nullifier matches public input
            nullifier.enforce_equal(&input_nullifiers[i])?;

            // SECURITY: Check if amount is zero (for conditional Merkle proof check)
            let amount_is_zero = in_amount.is_eq(&zero)?;

            // SECURITY: Range check - ensure input amount fits in 248 bits
            // This prevents overflow attacks in the first transaction
            let amount_bits = in_amount.to_bits_le()?;
            if amount_bits.len() < 254 {
                return Err(r1cs::SynthesisError::Unsatisfiable);
            }

            // Enforce top 6 bits (248-253) are zero when amount is non-zero
            // When amount is zero, this constraint is automatically satisfied
            for bit in &amount_bits[248..254] {
                let bit_should_be_zero = bit.clone().not();
                let condition = &amount_is_zero | &bit_should_be_zero;
                condition.enforce_equal(&Boolean::constant(true))?;
            }

            // SECURITY: Verify Merkle proof only if amount is non-zero
            // This optimization reduces constraints for zero-value inputs
            let merkle_path = merkle_paths[i].clone();
            let merkle_path_membership =
                merkle_path.check_membership(&root, &commitment, &hasher)?;

            // FIXED: Use conditional enforcement to reduce constraints
            // Only enforce Merkle membership when amount is non-zero
            // When amount_is_zero = true, this is automatically satisfied
            // When amount_is_zero = false, membership must be true
            merkle_path_membership.conditional_enforce_equal(
                &Boolean::constant(true),
                &amount_is_zero.clone().not(),
            )?;

            sum_ins += &in_amount;
        }

        // ============================================
        // VERIFY OUTPUT UTXOs
        // ============================================
        let mut sum_outs = FpVar::<Fr>::zero();

        let out_public_key = [
            FpVar::new_witness(ns!(cs, "out_public_key_0"), || Ok(self.out_public_keys[0]))?,
            FpVar::new_witness(ns!(cs, "out_public_key_1"), || Ok(self.out_public_keys[1]))?,
        ];

        let out_amounts = [
            FpVar::new_witness(ns!(cs, "out_amount_0"), || Ok(self.out_amounts[0]))?,
            FpVar::new_witness(ns!(cs, "out_amount_1"), || Ok(self.out_amounts[1]))?,
        ];

        let out_blindings = [
            FpVar::new_witness(ns!(cs, "out_blinding_0"), || Ok(self.out_blindings[0]))?,
            FpVar::new_witness(ns!(cs, "out_blinding_1"), || Ok(self.out_blindings[1]))?,
        ];

        for i in 0..N_OUTS {
            let out_amount = out_amounts[i].clone();

            // Calculate output commitment: commitment = Poseidon3(amount, pubkey, blinding)
            let expected_commitment =
                hasher.hash3(&out_amount, &out_public_key[i], &out_blindings[i])?;

            // Enforce computed commitment matches public input
            expected_commitment.enforce_equal(&output_commitment[i])?;

            // SECURITY: Range check - ensure output amount fits in 248 bits
            // This prevents overflow attacks
            let amount_is_zero = out_amount.is_eq(&zero)?;
            let amount_bits = out_amount.to_bits_le()?;
            if amount_bits.len() < 254 {
                return Err(r1cs::SynthesisError::Unsatisfiable);
            }

            // Enforce top 6 bits (248-253) are zero when amount is non-zero
            // When amount is zero, this constraint is automatically satisfied
            for bit in &amount_bits[248..254] {
                let bit_should_be_zero = bit.clone().not();
                let condition = &amount_is_zero | &bit_should_be_zero;
                condition.enforce_equal(&Boolean::constant(true))?;
            }

            sum_outs += &out_amount;
        }

        // ============================================
        // VERIFY NO DUPLICATE NULLIFIERS
        // ============================================
        // SECURITY: Prevent using same nullifier twice in one transaction
        // For N_INS = 2, we only need one comparison
        // FIXED: Use enforce_not_equal instead of is_neq for efficiency
        input_nullifiers[0].enforce_not_equal(&input_nullifiers[1])?;

        // ============================================
        // VERIFY AMOUNT CONSERVATION
        // ============================================
        // SECURITY: Ensure no value is created or destroyed
        // sum(inputs) + public_amount = sum(outputs)
        (sum_ins + public_amount).enforce_equal(&sum_outs)?;

        // ============================================
        // OPTIONAL SAFETY CONSTRAINT
        // ============================================
        // Create a constraint involving ext_data_hash to ensure it cannot be changed
        // This creates: ext_data_hash^2 = ext_data_square
        // The constraint itself isn't checked, but it prevents optimization that removes ext_data_hash
        let _ext_data_square = &ext_data_hash * &ext_data_hash;

        Ok(())
    }
}
