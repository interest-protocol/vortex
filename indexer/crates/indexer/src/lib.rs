use move_core_types::account_address::AccountAddress;
use std::str::FromStr;
use sui_types::base_types::ObjectID;

pub mod handlers;
pub mod models;
pub mod store;
pub mod traits;

/// Supported Sui networks
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SuiNetwork {
    Mainnet,
    Testnet,
    Devnet,
}

impl SuiNetwork {
    /// Get the checkpoint store URL for this network
    pub fn checkpoint_url(&self) -> &'static str {
        match self {
            SuiNetwork::Mainnet => "https://checkpoints.mainnet.sui.io",
            SuiNetwork::Testnet => "https://checkpoints.testnet.sui.io",
            SuiNetwork::Devnet => "https://checkpoints.devnet.sui.io",
        }
    }
}

impl std::str::FromStr for SuiNetwork {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "mainnet" => Ok(SuiNetwork::Mainnet),
            "testnet" => Ok(SuiNetwork::Testnet),
            "devnet" => Ok(SuiNetwork::Devnet),
            _ => Err(format!(
                "Unknown network '{}'. Use: mainnet, testnet, or devnet",
                s
            )),
        }
    }
}

impl std::fmt::Display for SuiNetwork {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SuiNetwork::Mainnet => write!(f, "mainnet"),
            SuiNetwork::Testnet => write!(f, "testnet"),
            SuiNetwork::Devnet => write!(f, "devnet"),
        }
    }
}

/// Parse a package address string into an ObjectID
pub fn parse_package_id(package: &str) -> anyhow::Result<ObjectID> {
    ObjectID::from_str(package)
        .map_err(|e| anyhow::anyhow!("Invalid package address '{}': {}", package, e))
}

/// Parse a package address string into an AccountAddress
pub fn parse_package_address(package: &str) -> anyhow::Result<AccountAddress> {
    AccountAddress::from_str(package)
        .map_err(|e| anyhow::anyhow!("Invalid package address '{}': {}", package, e))
}
