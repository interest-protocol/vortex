// src/poseidon/poseidon_optimized.rs
//
// Optimized Poseidon hash implementation for BN254 (circomlib compatible).
//
// This implements the optimized Poseidon algorithm that matches circomlibjs.
// The optimized variant uses sparse matrix multiplication during partial rounds
// for better performance, requiring additional precomputed matrices S and P.

pub mod poseidon_constants_opt;

use ark_bn254::Fr;
use ark_ff::Field;
use num_bigint::BigUint;
use num_traits::Num;

/// Optimized Poseidon hasher for circomlib compatibility
pub struct PoseidonOptimized {
    t: usize,
    n_rounds_f: usize,
    n_rounds_p: usize,
    c: Vec<Fr>,      // Round constants
    s: Vec<Fr>,      // Sparse matrix constants for partial rounds
    m: Vec<Vec<Fr>>, // MDS matrix
    p: Vec<Vec<Fr>>, // Pre-sparse matrix
}

impl PoseidonOptimized {
    /// Create hasher for t=2 (1 input)
    pub fn new_t2(c: Vec<Fr>, s: Vec<Fr>, m: Vec<Vec<Fr>>, p: Vec<Vec<Fr>>) -> Self {
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
    pub fn new_t3(c: Vec<Fr>, s: Vec<Fr>, m: Vec<Vec<Fr>>, p: Vec<Vec<Fr>>) -> Self {
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
    pub fn new_t4(c: Vec<Fr>, s: Vec<Fr>, m: Vec<Vec<Fr>>, p: Vec<Vec<Fr>>) -> Self {
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
            // Use split_at_mut to avoid borrow checker issues
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
}

// Helper to parse field element from decimal string
pub fn fr_from_str(s: &str) -> Fr {
    Fr::from(BigUint::from_str_radix(s, 10).expect("Failed to parse field element"))
}

/// Convenience functions for single-width hashers
pub fn hash1(x: &Fr) -> Fr {
    let (c, s, m, p) = poseidon_constants_opt::constants_t2();
    let hasher = PoseidonOptimized::new_t2(c, s, m, p);
    hasher.hash(&[*x])
}

pub fn hash2(x: &Fr, y: &Fr) -> Fr {
    let (c, s, m, p) = poseidon_constants_opt::constants_t3();
    let hasher = PoseidonOptimized::new_t3(c, s, m, p);
    hasher.hash(&[*x, *y])
}

pub fn hash3(x: &Fr, y: &Fr, z: &Fr) -> Fr {
    let (c, s, m, p) = poseidon_constants_opt::constants_t4();
    let hasher = PoseidonOptimized::new_t4(c, s, m, p);
    hasher.hash(&[*x, *y, *z])
}

#[cfg(test)]
mod tests {
    use super::*;

    // You'll need to generate these constants from poseidon-constants-opt.ts
    // This is a placeholder test structure
    #[test]
    fn test_optimized_poseidon() {
        // Load constants from poseidon_constants_opt.rs
        let (c, s, m, p) = poseidon_constants_opt::constants_t3();
        let hasher = PoseidonOptimized::new_t3(c, s, m, p);

        let x = Fr::from(1u64);
        let y = Fr::from(2u64);
        let hash = hasher.hash(&[x, y]);

        // Expected from TypeScript: 7853200120776062878684798364095072458815029376092732009249414926327459813530n
        assert_eq!(
            hash,
            fr_from_str(
                "7853200120776062878684798364095072458815029376092732009249414926327459813530"
            )
        );
    }
}
