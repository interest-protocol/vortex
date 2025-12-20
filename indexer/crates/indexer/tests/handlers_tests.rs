use sui_types::base_types::SuiAddress;
use vortex_indexer::handlers::{bytes_to_address, extract_coin_type, u256_to_hex};

#[test]
fn test_u256_to_hex() {
    let value = [0u8; 32];
    assert_eq!(
        u256_to_hex(&value),
        "0x0000000000000000000000000000000000000000000000000000000000000000"
    );

    let value = [0xff; 32];
    assert_eq!(
        u256_to_hex(&value),
        "0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff"
    );

    let mut value = [0u8; 32];
    value[31] = 0x42;
    assert_eq!(
        u256_to_hex(&value),
        "0x0000000000000000000000000000000000000000000000000000000000000042"
    );
}

#[test]
fn test_bytes_to_address() {
    let bytes = [0u8; 32];
    let addr = bytes_to_address(&bytes);
    assert_eq!(addr, SuiAddress::from_bytes(&[0u8; 32]).unwrap());

    let bytes = [0xff; 32];
    let addr = bytes_to_address(&bytes);
    assert_eq!(addr, SuiAddress::from_bytes(&[0xff; 32]).unwrap());
}

#[test]
fn test_extract_coin_type_valid() {
    assert_eq!(extract_coin_type("0x2::sui::SUI"), None);

    assert_eq!(
        extract_coin_type("SomeEvent<0x2::sui::SUI>"),
        Some("0x2::sui::SUI".to_string())
    );

    assert_eq!(
        extract_coin_type("Module::Event<0xabc::token::TOKEN>"),
        Some("0xabc::token::TOKEN".to_string())
    );

    assert_eq!(
        extract_coin_type("Event<address::module::Type<Inner>>"),
        Some("address::module::Type<Inner>".to_string())
    );
}

#[test]
fn test_extract_coin_type_empty() {
    assert_eq!(extract_coin_type(""), None);
    assert_eq!(extract_coin_type("NoAngleBrackets"), None);
}

#[test]
fn test_extract_coin_type_nested() {
    let result = extract_coin_type("Event<Type1><Type2>");
    assert!(result.is_some());
    assert!(result.unwrap().contains("Type1"));
}
