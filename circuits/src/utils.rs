use ark_bn254::Fr;
use ark_ff::{BigInteger, PrimeField};
use num_bigint::BigUint;
use num_traits::Num;
use sha2::{Digest, Sha256};

pub fn sha256_hash(input: &Fr) -> Fr {
    // SHA256 hash of the input, converted to field element
    let input_bytes = input.into_bigint().to_bytes_be();
    let mut hasher = Sha256::new();
    hasher.update(&input_bytes);
    let result = hasher.finalize();

    // Convert the first 32 bytes of SHA256 output to field element
    Fr::from_be_bytes_mod_order(&result[..32])
}

pub fn parse_address(address: String) -> Fr {
    let clean_address = address.strip_prefix("0x").unwrap_or(address.as_str());
    let recipient_bigint =
        BigUint::from_str_radix(clean_address, 16).expect("Failed to parse address");
    Fr::from(recipient_bigint)
}
