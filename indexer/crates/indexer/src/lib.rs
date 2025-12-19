use move_core_types::account_address::AccountAddress;
use std::str::FromStr;
use url::Url;

pub mod handlers;
pub mod models;
pub mod store;

pub const MAINNET_REMOTE_STORE_URL: &str = "https://checkpoints.mainnet.sui.io";
pub const TESTNET_REMOTE_STORE_URL: &str = "https://checkpoints.testnet.sui.io";
pub const DEVNET_REMOTE_STORE_URL: &str = "https://checkpoints.devnet.sui.io";

pub const MAINNET_STREAMING_URL: &str = "https://fullnode.mainnet.sui.io:443";
pub const TESTNET_STREAMING_URL: &str = "https://fullnode.testnet.sui.io:443";
pub const DEVNET_STREAMING_URL: &str = "https://fullnode.devnet.sui.io:443";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SuiNetwork {
    Mainnet,
    Testnet,
    Devnet,
}

impl SuiNetwork {
    #[must_use]
    pub fn remote_store_url(&self) -> Url {
        let url_str = match self {
            Self::Mainnet => MAINNET_REMOTE_STORE_URL,
            Self::Testnet => TESTNET_REMOTE_STORE_URL,
            Self::Devnet => DEVNET_REMOTE_STORE_URL,
        };
        Url::parse(url_str).expect("hardcoded URL should be valid")
    }

    #[must_use]
    pub fn streaming_url(&self) -> Url {
        let url_str = match self {
            Self::Mainnet => MAINNET_STREAMING_URL,
            Self::Testnet => TESTNET_STREAMING_URL,
            Self::Devnet => DEVNET_STREAMING_URL,
        };
        Url::parse(url_str).expect("hardcoded URL should be valid")
    }
}

impl FromStr for SuiNetwork {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "mainnet" => Ok(Self::Mainnet),
            "testnet" => Ok(Self::Testnet),
            "devnet" => Ok(Self::Devnet),
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
            Self::Mainnet => write!(f, "mainnet"),
            Self::Testnet => write!(f, "testnet"),
            Self::Devnet => write!(f, "devnet"),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct VortexEnv {
    pub network: SuiNetwork,
    pub package_address: AccountAddress,
}

impl VortexEnv {
    #[must_use]
    pub const fn new(network: SuiNetwork, package_address: AccountAddress) -> Self {
        Self {
            network,
            package_address,
        }
    }

    #[must_use]
    pub fn remote_store_url(&self) -> Url {
        self.network.remote_store_url()
    }

    #[must_use]
    pub fn streaming_url(&self) -> Url {
        self.network.streaming_url()
    }
}

pub fn parse_package_address(package: &str) -> anyhow::Result<AccountAddress> {
    AccountAddress::from_str(package)
        .map_err(|e| anyhow::anyhow!("Invalid package address '{}': {}", package, e))
}
