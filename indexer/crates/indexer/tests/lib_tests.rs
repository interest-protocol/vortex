use std::str::FromStr;
use sui_types::base_types::SuiAddress;
use vortex_indexer::{parse_package_address, SuiNetwork, VortexEnv};

#[test]
fn sui_network_from_str_valid() {
    assert_eq!(
        "mainnet".parse::<SuiNetwork>().unwrap(),
        SuiNetwork::Mainnet
    );
    assert_eq!(
        "testnet".parse::<SuiNetwork>().unwrap(),
        SuiNetwork::Testnet
    );
    assert_eq!("devnet".parse::<SuiNetwork>().unwrap(), SuiNetwork::Devnet);
    assert_eq!(
        "MAINNET".parse::<SuiNetwork>().unwrap(),
        SuiNetwork::Mainnet
    );
    assert_eq!(
        "TestNet".parse::<SuiNetwork>().unwrap(),
        SuiNetwork::Testnet
    );
}

#[test]
fn sui_network_from_str_invalid() {
    assert!("invalid".parse::<SuiNetwork>().is_err());
    assert!("main".parse::<SuiNetwork>().is_err());
    assert!("".parse::<SuiNetwork>().is_err());
}

#[test]
fn sui_network_display() {
    assert_eq!(SuiNetwork::Mainnet.to_string(), "mainnet");
    assert_eq!(SuiNetwork::Testnet.to_string(), "testnet");
    assert_eq!(SuiNetwork::Devnet.to_string(), "devnet");
}

#[test]
fn sui_network_remote_store_url() {
    assert_eq!(
        SuiNetwork::Mainnet.remote_store_url().as_str(),
        "https://checkpoints.mainnet.sui.io/"
    );
    assert_eq!(
        SuiNetwork::Testnet.remote_store_url().as_str(),
        "https://checkpoints.testnet.sui.io/"
    );
    assert_eq!(
        SuiNetwork::Devnet.remote_store_url().as_str(),
        "https://checkpoints.devnet.sui.io/"
    );
}

#[test]
fn sui_network_streaming_url() {
    let mainnet_url = SuiNetwork::Mainnet.streaming_url();
    assert!(mainnet_url
        .as_str()
        .starts_with("https://fullnode.mainnet.sui.io"));

    let testnet_url = SuiNetwork::Testnet.streaming_url();
    assert!(testnet_url
        .as_str()
        .starts_with("https://fullnode.testnet.sui.io"));

    let devnet_url = SuiNetwork::Devnet.streaming_url();
    assert!(devnet_url
        .as_str()
        .starts_with("https://fullnode.devnet.sui.io"));
}

#[test]
fn vortex_env_new() {
    let addr = SuiAddress::from_bytes([1u8; 32]).unwrap();
    let env = VortexEnv::new(SuiNetwork::Mainnet, addr);
    assert_eq!(env.network, SuiNetwork::Mainnet);
    assert_eq!(env.package_address, addr);
}

#[test]
fn vortex_env_urls() {
    let addr = SuiAddress::from_bytes([0u8; 32]).unwrap();
    let env = VortexEnv::new(SuiNetwork::Testnet, addr);
    assert_eq!(
        env.remote_store_url().as_str(),
        "https://checkpoints.testnet.sui.io/"
    );
    assert!(env
        .streaming_url()
        .as_str()
        .starts_with("https://fullnode.testnet.sui.io"));
}

#[test]
fn parse_package_address_valid() {
    let addr = "0x0000000000000000000000000000000000000000000000000000000000000001";
    let result = parse_package_address(addr);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), SuiAddress::from_str(addr).unwrap());
}

#[test]
fn parse_package_address_invalid() {
    assert!(parse_package_address("invalid").is_err());
    assert!(parse_package_address("").is_err());
    assert!(parse_package_address("not_hex").is_err());
}

#[test]
fn parse_package_address_error_message() {
    let result = parse_package_address("bad_address");
    assert!(result.is_err());
    let err_msg = result.unwrap_err().to_string();
    assert!(err_msg.contains("Invalid package address"));
    assert!(err_msg.contains("bad_address"));
}
