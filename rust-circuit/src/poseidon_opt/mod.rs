// src/poseidon_opt/mod.rs
//
// Optimized Poseidon hash implementation for BN254 (circomlib compatible).
//
// This implements the optimized Poseidon algorithm that matches circomlibjs.
// The optimized variant uses sparse matrix multiplication during partial rounds
// for better performance, requiring additional precomputed matrices S and P.
//
// This module provides both native computation and R1CS constraint generation.

pub mod poseidon_constants_opt;

use ark_bn254::Fr;
use ark_ff::Field;
use ark_r1cs_std::{
    alloc::{AllocVar, AllocationMode},
    fields::fp::FpVar,
    prelude::FieldVar,
};
use ark_relations::r1cs::{Namespace, SynthesisError};
use num_bigint::BigUint;
use num_traits::Num;
use std::borrow::Borrow;

// =============================================================================
// NATIVE IMPLEMENTATION
// =============================================================================

/// Optimized Poseidon hasher for circomlib compatibility (native computation)
#[derive(Clone)]
pub struct PoseidonOptimized {
    pub t: usize,
    pub n_rounds_f: usize,
    pub n_rounds_p: usize,
    pub c: Vec<Fr>,      // Round constants
    pub s: Vec<Fr>,      // Sparse matrix constants for partial rounds
    pub m: Vec<Vec<Fr>>, // MDS matrix
    pub p: Vec<Vec<Fr>>, // Pre-sparse matrix
}

impl PoseidonOptimized {
    /// Create hasher for t=2 (1 input)
    pub fn new_t2() -> Self {
        let (c, s, m, p) = poseidon_constants_opt::constants_t2();
        Self {
            t: 2,
            n_rounds_f: 8,
            n_rounds_p: 56,
            c,
            s,
            m,
            p,
        }
    }

    /// Create hasher for t=3 (2 inputs)
    pub fn new_t3() -> Self {
        let (c, s, m, p) = poseidon_constants_opt::constants_t3();
        Self {
            t: 3,
            n_rounds_f: 8,
            n_rounds_p: 57,
            c,
            s,
            m,
            p,
        }
    }

    /// Create hasher for t=4 (3 inputs)
    pub fn new_t4() -> Self {
        let (c, s, m, p) = poseidon_constants_opt::constants_t4();
        Self {
            t: 4,
            n_rounds_f: 8,
            n_rounds_p: 56,
            c,
            s,
            m,
            p,
        }
    }

    /// S-box: x^5
    #[inline]
    fn pow5(x: Fr) -> Fr {
        let x2 = x.square();
        let x4 = x2.square();
        x4 * x
    }

    /// Matrix-vector multiplication
    fn mix(&self, state: &[Fr], matrix: &[Vec<Fr>]) -> Vec<Fr> {
        let mut result = vec![Fr::from(0u64); self.t];
        #[allow(clippy::needless_range_loop)]
        for i in 0..self.t {
            for j in 0..self.t {
                result[i] += matrix[j][i] * state[j];
            }
        }
        result
    }

    /// Hash inputs using optimized Poseidon algorithm
    ///
    /// This matches the circomlibjs implementation exactly.
    pub fn hash(&self, inputs: &[Fr]) -> Fr {
        assert_eq!(
            inputs.len(),
            self.t - 1,
            "Wrong number of inputs for this hasher"
        );

        // Initialize state: [0, input1, input2, ...]
        let mut state = vec![Fr::from(0u64)];
        state.extend_from_slice(inputs);

        // Add initial round constants
        #[allow(clippy::needless_range_loop)]
        for i in 0..self.t {
            state[i] += self.c[i];
        }

        // First half of full rounds (minus 1)
        for r in 0..(self.n_rounds_f / 2 - 1) {
            // Apply S-box to all elements
            state = state.iter().map(|&x| Self::pow5(x)).collect();
            // Add round constants
            #[allow(clippy::needless_range_loop)]
            for i in 0..self.t {
                state[i] += self.c[(r + 1) * self.t + i];
            }
            // Mix with MDS matrix
            state = self.mix(&state, &self.m);
        }

        // Last round of first half (uses P matrix instead of M)
        state = state.iter().map(|&x| Self::pow5(x)).collect();
        #[allow(clippy::needless_range_loop)]
        for i in 0..self.t {
            state[i] += self.c[(self.n_rounds_f / 2 - 1 + 1) * self.t + i];
        }
        // Mix with pre-sparse matrix P
        state = self.mix(&state, &self.p);

        // Partial rounds (optimized sparse multiplication)
        for r in 0..self.n_rounds_p {
            // Apply S-box only to first element
            state[0] = Self::pow5(state[0]);
            // Add round constant only to first element
            state[0] += self.c[(self.n_rounds_f / 2 + 1) * self.t + r];

            // Sparse matrix multiplication
            // s0 = sum(S[r*stride + j] * state[j])
            let stride = self.t * 2 - 1;
            let mut s0 = Fr::from(0u64);
            #[allow(clippy::needless_range_loop)]
            for j in 0..self.t {
                s0 += self.s[stride * r + j] * state[j];
            }

            // state[k] += state[0] * S[r*stride + t + k - 1] for k in 1..t
            let state0 = state[0];
            #[allow(clippy::needless_range_loop)]
            for k in 1..self.t {
                state[k] += state0 * self.s[stride * r + self.t + k - 1];
            }
            state[0] = s0;
        }

        // Second half of full rounds (minus 1)
        for r in 0..(self.n_rounds_f / 2 - 1) {
            // Apply S-box to all elements
            state = state.iter().map(|&x| Self::pow5(x)).collect();
            // Add round constants
            #[allow(clippy::needless_range_loop)]
            for i in 0..self.t {
                state[i] +=
                    self.c[(self.n_rounds_f / 2 + 1) * self.t + self.n_rounds_p + r * self.t + i];
            }
            // Mix with MDS matrix
            state = self.mix(&state, &self.m);
        }

        // Final round (no round constants added after)
        state = state.iter().map(|&x| Self::pow5(x)).collect();
        state = self.mix(&state, &self.m);

        state[0]
    }

    /// Hash a single field element
    pub fn hash1(&self, x: &Fr) -> Fr {
        self.hash(&[*x])
    }

    /// Hash two field elements
    pub fn hash2(&self, x: &Fr, y: &Fr) -> Fr {
        self.hash(&[*x, *y])
    }

    /// Hash three field elements
    pub fn hash3(&self, x: &Fr, y: &Fr, z: &Fr) -> Fr {
        self.hash(&[*x, *y, *z])
    }
}

// =============================================================================
// CONSTRAINT GADGET IMPLEMENTATION
// =============================================================================

/// Constraint gadget for optimized Poseidon hash (R1CS compatible)
///
/// This generates constraints that match the optimized Poseidon algorithm,
/// ensuring compatibility with circomlib circuits.
#[derive(Clone)]
pub struct PoseidonOptimizedVar {
    pub t: usize,
    pub n_rounds_f: usize,
    pub n_rounds_p: usize,
    pub c: Vec<Fr>,
    pub s: Vec<Fr>,
    pub m: Vec<Vec<Fr>>,
    pub p: Vec<Vec<Fr>>,
}

impl PoseidonOptimizedVar {
    /// Create constraint gadget for t=2 (1 input)
    pub fn new_t2() -> Self {
        let (c, s, m, p) = poseidon_constants_opt::constants_t2();
        Self {
            t: 2,
            n_rounds_f: 8,
            n_rounds_p: 56,
            c,
            s,
            m,
            p,
        }
    }

    /// Create constraint gadget for t=3 (2 inputs)
    pub fn new_t3() -> Self {
        let (c, s, m, p) = poseidon_constants_opt::constants_t3();
        Self {
            t: 3,
            n_rounds_f: 8,
            n_rounds_p: 57,
            c,
            s,
            m,
            p,
        }
    }

    /// Create constraint gadget for t=4 (3 inputs)
    pub fn new_t4() -> Self {
        let (c, s, m, p) = poseidon_constants_opt::constants_t4();
        Self {
            t: 4,
            n_rounds_f: 8,
            n_rounds_p: 56,
            c,
            s,
            m,
            p,
        }
    }

    /// S-box as constraint: x^5
    #[inline]
    fn pow5_var(x: &FpVar<Fr>) -> Result<FpVar<Fr>, SynthesisError> {
        let x2 = x.square()?;
        let x4 = x2.square()?;
        Ok(&x4 * x)
    }

    /// Matrix-vector multiplication with FpVar
    fn mix_var(
        &self,
        state: &[FpVar<Fr>],
        matrix: &[Vec<Fr>],
    ) -> Result<Vec<FpVar<Fr>>, SynthesisError> {
        let mut result = Vec::with_capacity(self.t);
        for i in 0..self.t {
            let mut acc = FpVar::<Fr>::zero();
            for j in 0..self.t {
                acc += &state[j] * matrix[j][i];
            }
            result.push(acc);
        }
        Ok(result)
    }

    /// Hash with constraint generation - matches optimized algorithm exactly
    pub fn hash(&self, inputs: &[FpVar<Fr>]) -> Result<FpVar<Fr>, SynthesisError> {
        assert_eq!(
            inputs.len(),
            self.t - 1,
            "Wrong number of inputs for this hasher"
        );

        // Initialize state: [0, input1, input2, ...]
        let mut state = vec![FpVar::<Fr>::zero()];
        state.extend(inputs.iter().cloned());

        // Add initial round constants
        for (i, state_elem) in state.iter_mut().enumerate() {
            *state_elem += self.c[i];
        }

        // First half of full rounds (minus 1)
        for r in 0..(self.n_rounds_f / 2 - 1) {
            // Apply S-box to all elements
            let mut new_state = Vec::with_capacity(self.t);
            for s in &state {
                new_state.push(Self::pow5_var(s)?);
            }
            state = new_state;
            // Add round constants
            for (i, state_elem) in state.iter_mut().enumerate() {
                *state_elem += self.c[(r + 1) * self.t + i];
            }
            // Mix with MDS matrix
            state = self.mix_var(&state, &self.m)?;
        }

        // Last round of first half (uses P matrix instead of M)
        let mut new_state = Vec::with_capacity(self.t);
        for s in &state {
            new_state.push(Self::pow5_var(s)?);
        }
        state = new_state;
        for (i, state_elem) in state.iter_mut().enumerate() {
            *state_elem += self.c[(self.n_rounds_f / 2 - 1 + 1) * self.t + i];
        }
        // Mix with pre-sparse matrix P
        state = self.mix_var(&state, &self.p)?;

        // Partial rounds (optimized sparse multiplication)
        for r in 0..self.n_rounds_p {
            // Apply S-box only to first element
            state[0] = Self::pow5_var(&state[0])?;
            // Add round constant only to first element
            state[0] += self.c[(self.n_rounds_f / 2 + 1) * self.t + r];

            // Sparse matrix multiplication
            let stride = self.t * 2 - 1;
            let mut s0 = FpVar::<Fr>::zero();
            for (j, state_elem) in state.iter().enumerate() {
                s0 += state_elem * self.s[stride * r + j];
            }

            let state0 = state[0].clone();
            for (k, state_elem) in state.iter_mut().enumerate().skip(1) {
                *state_elem += &state0 * self.s[stride * r + self.t + k - 1];
            }
            state[0] = s0;
        }

        // Second half of full rounds (minus 1)
        for r in 0..(self.n_rounds_f / 2 - 1) {
            // Apply S-box to all elements
            let mut new_state = Vec::with_capacity(self.t);
            for s in &state {
                new_state.push(Self::pow5_var(s)?);
            }
            state = new_state;
            // Add round constants
            for (i, state_elem) in state.iter_mut().enumerate() {
                *state_elem +=
                    self.c[(self.n_rounds_f / 2 + 1) * self.t + self.n_rounds_p + r * self.t + i];
            }
            // Mix with MDS matrix
            state = self.mix_var(&state, &self.m)?;
        }

        // Final round (no round constants added after)
        let mut new_state = Vec::with_capacity(self.t);
        for s in &state {
            new_state.push(Self::pow5_var(s)?);
        }
        state = new_state;
        state = self.mix_var(&state, &self.m)?;

        Ok(state[0].clone())
    }

    /// Hash a single field element
    pub fn hash1(&self, x: &FpVar<Fr>) -> Result<FpVar<Fr>, SynthesisError> {
        self.hash(std::slice::from_ref(x))
    }

    /// Hash two field elements
    pub fn hash2(&self, x: &FpVar<Fr>, y: &FpVar<Fr>) -> Result<FpVar<Fr>, SynthesisError> {
        self.hash(&[x.clone(), y.clone()])
    }

    /// Hash three field elements
    pub fn hash3(
        &self,
        a: &FpVar<Fr>,
        b: &FpVar<Fr>,
        c: &FpVar<Fr>,
    ) -> Result<FpVar<Fr>, SynthesisError> {
        self.hash(&[a.clone(), b.clone(), c.clone()])
    }
}

/// Allow allocating PoseidonOptimizedVar as a constant in constraint systems
impl AllocVar<PoseidonOptimized, Fr> for PoseidonOptimizedVar {
    fn new_variable<T: Borrow<PoseidonOptimized>>(
        _cs: impl Into<Namespace<Fr>>,
        f: impl FnOnce() -> Result<T, SynthesisError>,
        _mode: AllocationMode,
    ) -> Result<Self, SynthesisError> {
        f().map(|param| {
            let p = param.borrow();
            Self {
                t: p.t,
                n_rounds_f: p.n_rounds_f,
                n_rounds_p: p.n_rounds_p,
                c: p.c.clone(),
                s: p.s.clone(),
                m: p.m.clone(),
                p: p.p.clone(),
            }
        })
    }
}

// =============================================================================
// CONVENIENCE FUNCTIONS
// =============================================================================

/// Helper to parse field element from decimal string
pub fn fr_from_str(s: &str) -> Fr {
    Fr::from(BigUint::from_str_radix(s, 10).expect("Failed to parse field element"))
}

/// Hash a single field element (native)
pub fn hash1(x: &Fr) -> Fr {
    PoseidonOptimized::new_t2().hash1(x)
}

/// Hash two field elements (native)
pub fn hash2(x: &Fr, y: &Fr) -> Fr {
    PoseidonOptimized::new_t3().hash2(x, y)
}

/// Hash three field elements (native)
pub fn hash3(x: &Fr, y: &Fr, z: &Fr) -> Fr {
    PoseidonOptimized::new_t4().hash3(x, y, z)
}

// =============================================================================
// TESTS
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use ark_r1cs_std::R1CSVar;
    use ark_relations::r1cs::ConstraintSystem;

    #[test]
    fn test_optimized_poseidon_t2() {
        let hasher = PoseidonOptimized::new_t2();

        let x = Fr::from(1u64);
        let hash = hasher.hash(&[x]);

        // Expected from TypeScript circomlibjs
        let expected = fr_from_str(
            "18586133768512220936620570745912940619677854269274689475585506675881198879027",
        );
        assert_eq!(hash, expected);
    }

    #[test]
    fn test_optimized_poseidon_t3() {
        let hasher = PoseidonOptimized::new_t3();

        let x = Fr::from(1u64);
        let y = Fr::from(2u64);
        let hash = hasher.hash(&[x, y]);

        // Expected from TypeScript circomlibjs
        let expected = fr_from_str(
            "7853200120776062878684798364095072458815029376092732009249414926327459813530",
        );
        assert_eq!(hash, expected);
    }

    #[test]
    fn test_optimized_poseidon_t4() {
        let hasher = PoseidonOptimized::new_t4();

        let x = Fr::from(1u64);
        let y = Fr::from(2u64);
        let z = Fr::from(3u64);
        let hash = hasher.hash(&[x, y, z]);

        // Expected from TypeScript circomlibjs
        let expected = fr_from_str(
            "6542985608222806190361240322586112750744169038454362455181422643027100751666",
        );
        assert_eq!(hash, expected);
    }

    #[test]
    fn test_constraint_gadget_matches_native() {
        let cs = ConstraintSystem::<Fr>::new_ref();

        let x = Fr::from(1u64);
        let y = Fr::from(2u64);

        // Native computation
        let native_hash = hash2(&x, &y);

        // Constraint computation
        let x_var = FpVar::new_witness(cs.clone(), || Ok(x)).unwrap();
        let y_var = FpVar::new_witness(cs.clone(), || Ok(y)).unwrap();

        let hasher_var = PoseidonOptimizedVar::new_t3();
        let hash_var = hasher_var.hash2(&x_var, &y_var).unwrap();

        // Check they match
        assert_eq!(hash_var.value().unwrap(), native_hash);

        // Check constraints are satisfied
        assert!(cs.is_satisfied().unwrap());
    }

    #[test]
    fn test_all_widths() {
        // t=2
        let h1 = hash1(&Fr::from(42u64));
        assert_ne!(h1, Fr::from(0u64));

        // t=3
        let h2 = hash2(&Fr::from(1u64), &Fr::from(2u64));
        assert_ne!(h2, Fr::from(0u64));

        // t=4
        let h3 = hash3(&Fr::from(1u64), &Fr::from(2u64), &Fr::from(3u64));
        assert_ne!(h3, Fr::from(0u64));
    }

    #[test]
    fn test_constraint_count() {
        let cs = ConstraintSystem::<Fr>::new_ref();

        let x = FpVar::new_witness(cs.clone(), || Ok(Fr::from(1u64))).unwrap();
        let y = FpVar::new_witness(cs.clone(), || Ok(Fr::from(2u64))).unwrap();

        let hasher = PoseidonOptimizedVar::new_t3();
        let _hash = hasher.hash2(&x, &y).unwrap();

        // Print constraint count for optimization analysis
        println!("Poseidon t=3 constraints: {}", cs.num_constraints());

        // Should have constraints (exact number depends on implementation)
        assert!(cs.num_constraints() > 0);
    }
}
