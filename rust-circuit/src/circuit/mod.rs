use crate::{
    constants::{LEVEL, MAX_AMOUNT_BITS, N_INS, N_OUTS},
    merkle_tree::{Path, PathVar},
    poseidon::{PoseidonHash, PoseidonHashVar},
};
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
///
/// # Commitment Scheme
///
/// - Input commitment: `Poseidon3(amount, pubkey, blinding)`
/// - Output commitment: `Poseidon3(amount, pubkey, blinding)`
/// - Nullifier: `Poseidon3(commitment, path_index, signature)`
/// - Signature: `Poseidon3(privkey, commitment, path_index)`
/// - Public key: `Poseidon1(privkey)`
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

    /// Creates a new circuit with validation.
    ///
    /// # Errors
    /// Returns error if:
    /// - Path indices exceed tree capacity (>= 2^LEVEL)
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        hasher: PoseidonHash,
        root: Fr,
        public_amount: Fr,
        ext_data_hash: Fr,
        input_nullifiers: [Fr; N_INS],
        output_commitment: [Fr; N_OUTS],
        in_private_keys: [Fr; N_INS],
        in_amounts: [Fr; N_INS],
        in_blindings: [Fr; N_INS],
        in_path_indices: [Fr; N_INS],
        merkle_paths: [Path<LEVEL>; N_INS],
        out_public_keys: [Fr; N_OUTS],
        out_amounts: [Fr; N_OUTS],
        out_blindings: [Fr; N_OUTS],
    ) -> anyhow::Result<Self> {
        // Validate path indices fit in tree
        let max_index = Fr::from(1u128 << LEVEL);
        for (i, idx) in in_path_indices.iter().enumerate() {
            if *idx >= max_index {
                return Err(anyhow::anyhow!(
                    "Input {} path index exceeds tree capacity (>= 2^{})",
                    i,
                    LEVEL
                ));
            }
        }

        Ok(Self {
            hasher,
            root,
            public_amount,
            ext_data_hash,
            input_nullifiers,
            output_commitment,
            in_private_keys,
            in_amounts,
            in_blindings,
            in_path_indices,
            merkle_paths,
            out_public_keys,
            out_amounts,
            out_blindings,
        })
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
        // Note: In Move, these are serialized as individual elements, not vectors
        // ============================================
        let root = FpVar::new_input(ns!(cs, "root"), || Ok(self.root))?;
        let public_amount = FpVar::new_input(ns!(cs, "public_amount"), || Ok(self.public_amount))?;
        // ext_data_hash is a public input that must be included in the proof
        // It's allocated here to ensure it's in the public input vector
        let _ext_data_hash = FpVar::new_input(ns!(cs, "ext_data_hash"), || Ok(self.ext_data_hash))?;

        // Individual public inputs (not arrays) to match Move contract serialization
        let input_nullifier_0 =
            FpVar::new_input(
                ns!(cs, "input_nullifier_0"),
                || Ok(self.input_nullifiers[0]),
            )?;
        let input_nullifier_1 =
            FpVar::new_input(
                ns!(cs, "input_nullifier_1"),
                || Ok(self.input_nullifiers[1]),
            )?;

        let output_commitment_0 = FpVar::new_input(ns!(cs, "output_commitment_0"), || {
            Ok(self.output_commitment[0])
        })?;
        let output_commitment_1 = FpVar::new_input(ns!(cs, "output_commitment_1"), || {
            Ok(self.output_commitment[1])
        })?;

        // Create arrays from individual variables for use in loops
        let input_nullifiers = [input_nullifier_0, input_nullifier_1];
        let output_commitment = [output_commitment_0, output_commitment_1];

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

        // Allocate output witnesses early (before input processing)
        // This improves constraint ordering and can help with optimization
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

        // ============================================
        // VERIFY INPUT UTXOs
        // ============================================
        let mut sum_ins = FpVar::<Fr>::zero();
        let zero = FpVar::<Fr>::zero();

        for i in 0..N_INS {
            // Derive public key from private key: pubkey = Poseidon1(privkey)
            let public_key = hasher.hash1(&in_private_key[i])?;

            // Calculate commitment: commitment = Poseidon3(amount, pubkey, blinding)
            let commitment = hasher.hash3(&in_amounts[i], &public_key, &in_blindings[i])?;

            // Calculate signature: sig = Poseidon3(privkey, commitment, path_index)
            let signature = hasher.hash3(&in_private_key[i], &commitment, &in_path_indices[i])?;

            // Calculate nullifier: nullifier = Poseidon3(commitment, path_index, signature)
            let nullifier = hasher.hash3(&commitment, &in_path_indices[i], &signature)?;

            // Enforce computed nullifier matches public input
            nullifier.enforce_equal(&input_nullifiers[i])?;

            // SECURITY: Check if amount is zero (for conditional Merkle proof check)
            let amount_is_zero = in_amounts[i].is_eq(&zero)?;

            // SECURITY: Range check - ensure input amount fits in MAX_AMOUNT_BITS
            // This prevents overflow attacks
            enforce_range_check(&in_amounts[i], &amount_is_zero)?;

            // SECURITY: Verify Merkle proof only if amount is non-zero
            // This optimization reduces constraints for zero-value inputs
            let merkle_path_membership =
                merkle_paths[i].check_membership(&root, &commitment, &hasher)?;

            // Only enforce Merkle membership when amount is non-zero
            let amount_is_non_zero = amount_is_zero.not();
            merkle_path_membership
                .conditional_enforce_equal(&Boolean::constant(true), &amount_is_non_zero)?;

            sum_ins += &in_amounts[i];
        }

        // ============================================
        // VERIFY OUTPUT UTXOs
        // ============================================
        let mut sum_outs = FpVar::<Fr>::zero();

        for i in 0..N_OUTS {
            // Calculate output commitment: commitment = Poseidon3(amount, pubkey, blinding)
            let expected_commitment =
                hasher.hash3(&out_amounts[i], &out_public_key[i], &out_blindings[i])?;

            // Enforce computed commitment matches public input
            expected_commitment.enforce_equal(&output_commitment[i])?;

            // SECURITY: Range check - ensure output amount fits in MAX_AMOUNT_BITS
            let amount_is_zero = out_amounts[i].is_eq(&zero)?;
            enforce_range_check(&out_amounts[i], &amount_is_zero)?;

            sum_outs += &out_amounts[i];
        }

        // ============================================
        // VERIFY NO DUPLICATE NULLIFIERS
        // ============================================
        // SECURITY: Prevent using same nullifier twice in one transaction
        //
        // Optimization: For N_INS=2, we only need 1 comparison (nullifiers[0] != nullifiers[1])
        // This is the minimal constraint set - exactly 1 enforce_not_equal constraint.
        //
        // Alternative approaches considered:
        // - Loop over all pairs: Same constraint count for N_INS=2, but adds loop overhead
        // - Product of differences: More expensive (requires multiplications)
        // - Direct check: Optimal for fixed N_INS=2, explicit and clear
        //
        // If N_INS changes in the future, generalize to: for i in 0..N_INS { for j in (i+1)..N_INS { ... } }
        input_nullifiers[0].enforce_not_equal(&input_nullifiers[1])?;

        // ============================================
        // VERIFY AMOUNT CONSERVATION
        // ============================================
        // SECURITY: Ensure no value is created or destroyed
        // sum(inputs) + public_amount = sum(outputs)
        (sum_ins + public_amount).enforce_equal(&sum_outs)?;

        Ok(())
    }
}

/// Optimized range check: ensures `value` < 2^MAX_AMOUNT_BITS
///
/// More efficient than Circom's Num2Bits approach: instead of reconstructing from 248 bits,
/// we only check that the upper 6 bits [248..254) are zero when value is non-zero.
/// This achieves the same security guarantee with far fewer constraints.
///
/// # Arguments
/// * `value` - The field element to range check
/// * `value_is_zero` - Boolean indicating if value is zero (skip check if true)
///
/// # Constraints
/// - Always: ~254 constraints for bit decomposition (unavoidable with ark_r1cs_std)
/// - When value_is_zero = true: Only bit decomposition, no range check constraints
/// - When value_is_zero = false: Bit decomposition + 6 conditional equality checks
///
/// # Note on Optimization
/// Unfortunately, ark_r1cs_std's `to_bits_le()` always performs full bit decomposition
/// (~254 constraints) regardless of whether we conditionally use the bits. The optimization
/// here is that we only enforce the 6 upper-bit checks when the value is non-zero, saving
/// 6 constraints for zero values. A more efficient implementation would require custom
/// bit decomposition that can be conditionally skipped entirely.
fn enforce_range_check(value: &FpVar<Fr>, value_is_zero: &Boolean<Fr>) -> r1cs::Result<()> {
    use ark_r1cs_std::prelude::ToBitsGadget;

    // Decompose value into bits (all 254 bits for BN254 field)
    // Note: This always creates ~254 constraints, even for zero values
    let value_bits = value.to_bits_le()?;
    let value_is_non_zero = value_is_zero.not();

    // Efficient approach: Check that bits [MAX_AMOUNT_BITS..254) are all zero when value is non-zero
    // For MAX_AMOUNT_BITS = 248, we check bits [248..254) = 6 bits
    // This is equivalent to Circom's Num2Bits(248) but more efficient:
    // - Circom: 248 multiplications + 248 additions + 1 equality check
    // - This: 6 conditional equality checks (only enforced when value is non-zero)
    for bit in value_bits
        .iter()
        .skip(MAX_AMOUNT_BITS)
        .take(254 - MAX_AMOUNT_BITS)
    {
        // Constraint: if value is non-zero, then bit must be zero
        // This is: NOT(value_is_zero) IMPLIES (bit == false)
        bit.conditional_enforce_equal(&Boolean::constant(false), &value_is_non_zero)?;
    }

    Ok(())
}
