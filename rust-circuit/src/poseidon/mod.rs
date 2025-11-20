// src/poseidon/mod.rs
//
// Poseidon hash implementation for BN254 using arkworks.
//
// This module uses constants from poseidon_constants.rs which must match
// the constants used in your Circom circuits and TypeScript SDK.
//
// The arkworks sponge implements the standard (non-optimized) Poseidon algorithm.
// If your other implementations use the optimized variant, ensure the constants
// produce compatible outputs or switch to matching implementations.

pub mod poseidon_constants;

use num_bigint::BigUint;
use num_traits::Num;
use std::borrow::Borrow;

use ark_bn254::Fr;
use ark_crypto_primitives::{
    crh::poseidon::constraints::CRHParametersVar,
    sponge::{
        constraints::CryptographicSpongeVar,
        poseidon::{constraints::PoseidonSpongeVar, PoseidonConfig, PoseidonSponge},
        CryptographicSponge,
    },
};
use ark_r1cs_std::{
    alloc::{AllocVar, AllocationMode},
    fields::fp::FpVar,
    R1CSVar,
};
use ark_relations::r1cs::{Namespace, SynthesisError};

use poseidon_constants::{FULL_ROUNDS, PARTIAL_ROUNDS};

pub fn get_hashers() -> (PoseidonHash, PoseidonHash, PoseidonHash) {
    (
        PoseidonHash::new(poseidon_bn254_t2()),
        PoseidonHash::new(poseidon_bn254_t3()),
        PoseidonHash::new(poseidon_bn254_t4()),
    )
}

/// Poseidon config for t=2 state (1 input + 1 capacity)
/// Partial rounds: 56
pub fn poseidon_bn254_t2() -> PoseidonConfig<Fr> {
    let (c_str, m_str) = poseidon_constants::constants();

    let width_idx = 0; // Index 0 for t=2
    let t = 2;
    let partial_rounds = PARTIAL_ROUNDS[width_idx];

    build_poseidon_config(
        &c_str[width_idx],
        &m_str[width_idx],
        t,
        FULL_ROUNDS,
        partial_rounds,
    )
}

/// Poseidon config for t=3 state (2 inputs + 1 capacity)
/// Partial rounds: 57
/// This is the most commonly used configuration.
pub fn poseidon_bn254_t3() -> PoseidonConfig<Fr> {
    let (c_str, m_str) = poseidon_constants::constants();

    let width_idx = 1; // Index 1 for t=3
    let t = 3;
    let partial_rounds = PARTIAL_ROUNDS[width_idx];

    build_poseidon_config(
        &c_str[width_idx],
        &m_str[width_idx],
        t,
        FULL_ROUNDS,
        partial_rounds,
    )
}

/// Poseidon config for t=4 state (3 inputs + 1 capacity)
/// Partial rounds: 56
pub fn poseidon_bn254_t4() -> PoseidonConfig<Fr> {
    let (c_str, m_str) = poseidon_constants::constants();

    let width_idx = 2; // Index 2 for t=4
    let t = 4;
    let partial_rounds = PARTIAL_ROUNDS[width_idx];

    build_poseidon_config(
        &c_str[width_idx],
        &m_str[width_idx],
        t,
        FULL_ROUNDS,
        partial_rounds,
    )
}

/// Build a PoseidonConfig from string constants.
///
/// # Arguments
/// * `round_constants` - Flat vector of round constants, will be reshaped to [round][state_idx]
/// * `mds_matrix` - MDS matrix as Vec<Vec<str>>
/// * `t` - State width (rate + capacity)
/// * `full_rounds` - Number of full rounds (typically 8)
/// * `partial_rounds` - Number of partial rounds (depends on t)
fn build_poseidon_config(
    round_constants: &[&str],
    mds_matrix: &[Vec<&str>],
    t: usize,
    full_rounds: usize,
    partial_rounds: usize,
) -> PoseidonConfig<Fr> {
    let total_rounds = full_rounds + partial_rounds;

    // Convert round constants from flat vector to ark format: Vec<Vec<Fr>>
    // arkworks expects ark[round][state_element]
    // Constants are provided as flat array, chunk by t for each round
    let ark: Vec<Vec<Fr>> = round_constants
        .chunks(t)
        .take(total_rounds)
        .map(|chunk| {
            chunk
                .iter()
                .map(|s| {
                    Fr::from(
                        BigUint::from_str_radix(s, 10).expect("Failed to parse round constant"),
                    )
                })
                .collect()
        })
        .collect();

    // Ensure we have enough rounds (pad if needed)
    let mut ark_padded = ark;
    while ark_padded.len() < total_rounds {
        ark_padded.push(vec![Fr::from(0u64); t]);
    }

    // Convert MDS matrix
    let mds: Vec<Vec<Fr>> = mds_matrix
        .iter()
        .map(|row| {
            row.iter()
                .map(|s| {
                    Fr::from(
                        BigUint::from_str_radix(s, 10).expect("Failed to parse MDS matrix element"),
                    )
                })
                .collect()
        })
        .collect();

    PoseidonConfig::<Fr> {
        full_rounds,
        partial_rounds,
        alpha: 5,
        ark: ark_padded,
        mds,
        rate: t - 1,
        capacity: 1,
    }
}

/// Native Poseidon hash (BN254) with 1-, 2- and 3-input helpers.
///
/// Uses constants from poseidon_constants.rs for cross-platform consistency
/// with the TypeScript SDK and Circom circuits.
#[derive(Debug, Clone)]
pub struct PoseidonHash {
    pub config: PoseidonConfig<Fr>,
}

impl PoseidonHash {
    pub fn new(config: PoseidonConfig<Fr>) -> Self {
        Self { config }
    }

    /// Hash a single field element
    ///
    /// # Panics
    /// Panics if the config is not for t=2 (1 input). Use `poseidon_bn254_t2()` to create the config.
    pub fn hash1(&self, x: &Fr) -> Fr {
        assert_eq!(
            self.config.rate, 1,
            "hash1 requires t=2 config (rate=1), but got rate={}. Use poseidon_bn254_t2().",
            self.config.rate
        );
        let mut sponge = PoseidonSponge::new(&self.config);
        sponge.absorb(x);
        let out = sponge.squeeze_field_elements::<Fr>(1);
        out[0]
    }

    /// Hash two field elements
    ///
    /// # Panics
    /// Panics if the config is not for t=3 (2 inputs). Use `poseidon_bn254_t3()` or `poseidon_bn254()` to create the config.
    pub fn hash2(&self, left: &Fr, right: &Fr) -> Fr {
        assert_eq!(
            self.config.rate, 2,
            "hash2 requires t=3 config (rate=2), but got rate={}. Use poseidon_bn254_t3() or poseidon_bn254().",
            self.config.rate
        );
        let mut sponge = PoseidonSponge::new(&self.config);
        sponge.absorb(left);
        sponge.absorb(right);
        let out = sponge.squeeze_field_elements::<Fr>(1);
        out[0]
    }

    /// Hash three field elements
    ///
    /// # Panics
    /// Panics if the config is not for t=4 (3 inputs). Use `poseidon_bn254_t4()` to create the config.
    pub fn hash3(&self, a: &Fr, b: &Fr, c: &Fr) -> Fr {
        assert_eq!(
            self.config.rate, 3,
            "hash3 requires t=4 config (rate=3), but got rate={}. Use poseidon_bn254_t4().",
            self.config.rate
        );
        let mut sponge = PoseidonSponge::new(&self.config);
        sponge.absorb(a);
        sponge.absorb(b);
        sponge.absorb(c);
        let out = sponge.squeeze_field_elements::<Fr>(1);
        out[0]
    }
}

/// Constraint gadget for Poseidon hash (BN254) with 1-, 2-, and 3-input helpers.
pub struct PoseidonHashVar {
    pub config: CRHParametersVar<Fr>,
}

impl PoseidonHashVar {
    pub fn hash1(&self, x: &FpVar<Fr>) -> Result<FpVar<Fr>, SynthesisError> {
        assert_eq!(
            self.config.parameters.rate, 1,
            "hash1 requires t=2 config (rate=1), but got rate={}",
            self.config.parameters.rate
        );

        let cs = x.cs();

        // All constants: compute natively and return a constant var.
        if cs.is_none() {
            let nx = x.value()?;
            let native = PoseidonHash::new(self.config.parameters.clone()).hash1(&nx);
            return Ok(FpVar::Constant(native));
        }

        // At least one witness: use sponge gadget.
        let mut sponge = PoseidonSpongeVar::new(cs, &self.config.parameters);
        sponge.absorb(x)?;
        let out = sponge.squeeze_field_elements(1)?;
        Ok(out[0].clone())
    }

    pub fn hash2(&self, left: &FpVar<Fr>, right: &FpVar<Fr>) -> Result<FpVar<Fr>, SynthesisError> {
        assert_eq!(
            self.config.parameters.rate, 2,
            "hash2 requires t=3 config (rate=2), but got rate={}",
            self.config.parameters.rate
        );

        let cs = left.cs().or(right.cs());

        if cs.is_none() {
            let native_left = left.value()?;
            let native_right = right.value()?;

            let native = PoseidonHash::new(self.config.parameters.clone())
                .hash2(&native_left, &native_right);

            return Ok(FpVar::Constant(native));
        }

        let mut sponge = PoseidonSpongeVar::new(cs, &self.config.parameters);
        sponge.absorb(left)?;
        sponge.absorb(right)?;
        let out = sponge.squeeze_field_elements(1)?;
        Ok(out[0].clone())
    }

    pub fn hash3(
        &self,
        a: &FpVar<Fr>,
        b: &FpVar<Fr>,
        c: &FpVar<Fr>,
    ) -> Result<FpVar<Fr>, SynthesisError> {
        assert_eq!(
            self.config.parameters.rate, 3,
            "hash3 requires t=4 config (rate=3), but got rate={}",
            self.config.parameters.rate
        );

        let cs = a.cs().or(b.cs()).or(c.cs());

        if cs.is_none() {
            let na = a.value()?;
            let nb = b.value()?;
            let nc = c.value()?;

            let native = PoseidonHash::new(self.config.parameters.clone()).hash3(&na, &nb, &nc);

            return Ok(FpVar::Constant(native));
        }

        let mut sponge = PoseidonSpongeVar::new(cs, &self.config.parameters);
        sponge.absorb(a)?;
        sponge.absorb(b)?;
        sponge.absorb(c)?;
        let out = sponge.squeeze_field_elements(1)?;
        Ok(out[0].clone())
    }
}

/// Allocate PoseidonHashVar from a PoseidonHash (native wrapper).
impl AllocVar<PoseidonHash, Fr> for PoseidonHashVar {
    fn new_variable<T: Borrow<PoseidonHash>>(
        cs: impl Into<Namespace<Fr>>,
        f: impl FnOnce() -> Result<T, SynthesisError>,
        mode: AllocationMode,
    ) -> Result<Self, SynthesisError> {
        f().and_then(|param| {
            let parameters = param.borrow();
            let cfg_var = CRHParametersVar::new_variable(cs, || Ok(&parameters.config), mode)?;
            Ok(Self { config: cfg_var })
        })
    }
}

/// Allocate PoseidonHashVar directly from a PoseidonConfig<Fr>.
impl AllocVar<PoseidonConfig<Fr>, Fr> for PoseidonHashVar {
    fn new_variable<T: Borrow<PoseidonConfig<Fr>>>(
        cs: impl Into<Namespace<Fr>>,
        f: impl FnOnce() -> Result<T, SynthesisError>,
        mode: AllocationMode,
    ) -> Result<Self, SynthesisError> {
        f().and_then(|param| {
            let cfg = param.borrow();
            let cfg_var = CRHParametersVar::new_variable(cs, || Ok(cfg), mode)?;
            Ok(Self { config: cfg_var })
        })
    }
}
