use ark_bn254::Fr;
use num_bigint::BigUint;
use num_traits::Num;

pub fn parse_address(address: String) -> Fr {
    let clean_address = address.strip_prefix("0x").unwrap_or(address.as_str());
    let recipient_bigint =
        BigUint::from_str_radix(clean_address, 16).expect("Failed to parse address");
    Fr::from(recipient_bigint)
}
