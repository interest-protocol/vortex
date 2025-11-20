// src/poseidon_bn254.rs

use std::borrow::Borrow;

use ark_bn254::Fr as ArkFr;
use ark_crypto_primitives::{
    crh::poseidon::constraints::CRHParametersVar,
    sponge::{
        constraints::CryptographicSpongeVar,
        poseidon::{constraints::PoseidonSpongeVar, PoseidonConfig, PoseidonSponge},
        CryptographicSponge,
    },
};
use ark_ff::BigInteger;
use ark_ff::PrimeField as ArkPrimeField;
use ark_r1cs_std::{
    alloc::{AllocVar, AllocationMode},
    fields::fp::FpVar,
    R1CSVar,
};
use ark_relations::r1cs::{Namespace, SynthesisError};
use ff::PrimeField;
use neptune::poseidon::HashMode::OptimizedStatic;
use neptune::Poseidon;
use num_bigint::BigUint;
use num_traits::Num;
use once_cell::sync::Lazy;

mod poseidon_constants;
use poseidon_constants::*;

/// Poseidon configuration for BN254
/// Loads constants from poseidon_constants.rs (lazy-loaded)
/// Note: This is for the old ark-crypto-primitives implementation (constraint system)
/// The new neptune-based implementation uses poseidon() function (native implementation)
/// Helper to create PoseidonConfig for a given width
fn create_poseidon_config(width_index: usize) -> PoseidonConfig<ArkFr> {
    let (constants_strings, matrices_strings) = poseidon_constants::constants();

    // Width is width_index + 1
    // width_index 0 -> width 2 (1 input, rate=1, capacity=1)
    // width_index 1 -> width 3 (2 inputs, rate=2, capacity=1)
    // width_index 2 -> width 4 (3 inputs, rate=3, capacity=1)
    let width = width_index + 2;
    let rate = width - 1;
    let capacity = 1;

    let round_constants_str = &constants_strings[width_index];
    let mds_matrix_str = &matrices_strings[width_index];
    let partial_rounds = [56, 57, 56][width_index]; // partial_rounds for widths 1, 2, 3

    // Convert round constants from strings directly to ark_bn254::Fr
    // Each round needs `width` constants, so we chunk the constants array
    let ark: Vec<Vec<ArkFr>> = round_constants_str
        .chunks(width)
        .map(|chunk| {
            chunk
                .iter()
                .map(|s| {
                    // Convert from decimal string to ark_bn254::Fr
                    let big_uint = BigUint::from_str_radix(s, 10)
                        .expect("Failed to parse poseidon constant as decimal");
                    ArkFr::from(big_uint)
                })
                .collect()
        })
        .collect();

    // Convert MDS matrix from strings directly to ark_bn254::Fr
    let mds: Vec<Vec<ArkFr>> = mds_matrix_str
        .iter()
        .map(|row| {
            row.iter()
                .map(|s| {
                    let big_uint = BigUint::from_str_radix(s, 10)
                        .expect("Failed to parse poseidon MDS matrix element as decimal");
                    ArkFr::from(big_uint)
                })
                .collect()
        })
        .collect();

    PoseidonConfig::<ArkFr> {
        full_rounds: 8,
        partial_rounds,
        alpha: 5,
        ark,
        mds,
        rate,
        capacity,
    }
}

/// Lazy-loaded Poseidon configurations for BN254 (for constraint system)
static POSEIDON_CONFIG_BN254_WIDTH2: Lazy<PoseidonConfig<ArkFr>> =
    Lazy::new(|| create_poseidon_config(0));
static POSEIDON_CONFIG_BN254_WIDTH3: Lazy<PoseidonConfig<ArkFr>> =
    Lazy::new(|| create_poseidon_config(1));
static POSEIDON_CONFIG_BN254_WIDTH4: Lazy<PoseidonConfig<ArkFr>> =
    Lazy::new(|| create_poseidon_config(2));

/// Poseidon configuration for BN254 (width 3, for backward compatibility)
pub fn poseidon_bn254() -> PoseidonConfig<ArkFr> {
    POSEIDON_CONFIG_BN254_WIDTH3.clone()
}

/// Native Poseidon hash (BN254) with 1-, 2- and 3-input helpers.
#[derive(Debug, Clone)]
pub struct PoseidonHash {
    pub config: PoseidonConfig<ArkFr>,
}

impl PoseidonHash {
    pub fn new(config: PoseidonConfig<ArkFr>) -> Self {
        Self { config }
    }

    pub fn hash1(&self, x: &ArkFr) -> ArkFr {
        // Use width 2 config for 1 input
        let config = POSEIDON_CONFIG_BN254_WIDTH2.clone();
        let mut sponge = PoseidonSponge::new(&config);
        sponge.absorb(x);
        let out = sponge.squeeze_field_elements::<ArkFr>(1);
        out[0]
    }

    pub fn hash2(&self, left: &ArkFr, right: &ArkFr) -> ArkFr {
        // Use width 3 config for 2 inputs
        let config = POSEIDON_CONFIG_BN254_WIDTH3.clone();
        let mut sponge = PoseidonSponge::new(&config);
        sponge.absorb(left);
        sponge.absorb(right);
        let out = sponge.squeeze_field_elements::<ArkFr>(1);
        out[0]
    }

    pub fn hash3(&self, a: &ArkFr, b: &ArkFr, c: &ArkFr) -> ArkFr {
        // Use width 4 config for 3 inputs
        let config = POSEIDON_CONFIG_BN254_WIDTH4.clone();
        let mut sponge = PoseidonSponge::new(&config);
        sponge.absorb(a);
        sponge.absorb(b);
        sponge.absorb(c);
        let out = sponge.squeeze_field_elements::<ArkFr>(1);
        out[0]
    }
}

/// Constraint gadget for Poseidon hash (BN254) with 1-, 2-, and 3-input helpers.
pub struct PoseidonHashVar {
    pub config: CRHParametersVar<ArkFr>,
}

impl PoseidonHashVar {
    pub fn hash1(&self, x: &FpVar<ArkFr>) -> Result<FpVar<ArkFr>, SynthesisError> {
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

    pub fn hash2(
        &self,
        left: &FpVar<ArkFr>,
        right: &FpVar<ArkFr>,
    ) -> Result<FpVar<ArkFr>, SynthesisError> {
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
        a: &FpVar<ArkFr>,
        b: &FpVar<ArkFr>,
        c: &FpVar<ArkFr>,
    ) -> Result<FpVar<ArkFr>, SynthesisError> {
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
impl AllocVar<PoseidonHash, ArkFr> for PoseidonHashVar {
    fn new_variable<T: Borrow<PoseidonHash>>(
        cs: impl Into<Namespace<ArkFr>>,
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

/// Allocate PoseidonHashVar directly from a PoseidonConfig<ArkFr>.
impl AllocVar<PoseidonConfig<ArkFr>, ArkFr> for PoseidonHashVar {
    fn new_variable<T: Borrow<PoseidonConfig<ArkFr>>>(
        cs: impl Into<Namespace<ArkFr>>,
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

// ============================================================================
// Helper functions for native hashing
// ============================================================================

/// Hash one field element using Poseidon
pub fn poseidon_hash1(x: ArkFr) -> Result<ArkFr, String> {
    poseidon(&[x])
}

/// Hash two field elements using Poseidon
pub fn poseidon_hash2(left: ArkFr, right: ArkFr) -> Result<ArkFr, String> {
    poseidon(&[left, right])
}

/// Hash three field elements using Poseidon
pub fn poseidon_hash3(a: ArkFr, b: ArkFr, c: ArkFr) -> Result<ArkFr, String> {
    poseidon(&[a, b, c])
}

/// Hash an array/slice of field elements using Poseidon (supports up to 3 inputs)
/// Uses neptune library exactly like fastcrypto-zkp does
pub fn poseidon(elements: &[ArkFr]) -> Result<ArkFr, String> {
    if elements.is_empty() {
        return Err("Cannot hash empty array".to_string());
    }
    if elements.len() > 3 {
        return Err("Cannot hash more than 3 elements".to_string());
    }

    // Convert ark_bn254::Fr to blstrs::Scalar for neptune
    // neptune uses ff::PrimeField, and blstrs::Scalar implements it for BN254

    // Match on input length like fastcrypto-zkp does
    let result = match elements.len() {
        1 => {
            let mut poseidon = Poseidon::new(&POSEIDON_CONSTANTS_U1);
            poseidon.reset();
            for input in elements.iter() {
                let fr = convert_ark_to_ff(input)?;
                poseidon
                    .input(fr)
                    .map_err(|e| format!("Neptune input error: {:?}", e))?;
            }
            poseidon.hash_in_mode(OptimizedStatic);
            poseidon.elements[0]
        }
        2 => {
            let mut poseidon = Poseidon::new(&POSEIDON_CONSTANTS_U2);
            poseidon.reset();
            for input in elements.iter() {
                let fr = convert_ark_to_ff(input)?;
                poseidon
                    .input(fr)
                    .map_err(|e| format!("Neptune input error: {:?}", e))?;
            }
            poseidon.hash_in_mode(OptimizedStatic);
            poseidon.elements[0]
        }
        3 => {
            let mut poseidon = Poseidon::new(&POSEIDON_CONSTANTS_U3);
            poseidon.reset();
            for input in elements.iter() {
                let fr = convert_ark_to_ff(input)?;
                poseidon
                    .input(fr)
                    .map_err(|e| format!("Neptune input error: {:?}", e))?;
            }
            poseidon.hash_in_mode(OptimizedStatic);
            poseidon.elements[0]
        }
        _ => return Err(format!("Unsupported number of inputs: {}", elements.len())),
    };

    // Convert back from ff::Fr to ark_bn254::Fr
    convert_ff_to_ark(&result)
}

/// Convert ark_bn254::Fr to blstrs::Scalar (BN254 field, implements ff::PrimeField)
/// Matches fastcrypto-zkp's field_element_to_fr conversion
fn convert_ark_to_ff(fr: &ArkFr) -> Result<blstrs::Scalar, String> {
    // fastcrypto-zkp uses: bytes.clone_from_slice(&fr.0.into_bigint().to_bytes_be());
    // then: crate::Fr::from_repr_vartime(FrRepr(bytes))
    // fastcrypto-zkp's Fr has #[PrimeFieldReprEndianness = "big"]
    // blstrs::Scalar might also use big-endian, let's try without reversal first
    use ff::PrimeField;
    let mut bytes = [0u8; 32];
    let fr_bytes = fr.into_bigint().to_bytes_be();
    if fr_bytes.len() > 32 {
        return Err("Field element too large".to_string());
    }
    // Copy to the end of the 32-byte array (big-endian)
    bytes[32 - fr_bytes.len()..].copy_from_slice(&fr_bytes);
    blstrs::Scalar::from_repr_vartime(<blstrs::Scalar as PrimeField>::Repr::from(bytes))
        .ok_or_else(|| "Failed to convert to blstrs::Scalar".to_string())
}

/// Convert blstrs::Scalar to ark_bn254::Fr
/// Matches fastcrypto-zkp's fr_to_field_element conversion
fn convert_ff_to_ark(fr: &blstrs::Scalar) -> Result<ArkFr, String> {
    // fastcrypto-zkp uses: Fr::from_be_bytes_mod_order(fr.to_repr().as_byte_slice())
    // fastcrypto-zkp's Fr has #[PrimeFieldReprEndianness = "big"], so to_repr() returns big-endian
    // If blstrs::Scalar also uses big-endian, we don't need to reverse
    let repr = fr.to_repr();
    let bytes: &[u8] = repr.as_ref();
    Ok(ArkFr::from_be_bytes_mod_order(bytes))
}
