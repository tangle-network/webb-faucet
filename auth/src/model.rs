use chrono::{DateTime, Utc};
use ethers_core::types::Address;
use sp_core::crypto::{AccountId32, Ss58Codec};

#[derive(
    Copy,
    Clone,
    Debug,
    Eq,
    Hash,
    PartialEq,
    Default,
    serde::Serialize,
    serde::Deserialize,
)]
#[serde(tag = "type", content = "value")]
pub enum UniversalWalletAddress {
    #[default]
    Unknown,
    Ethereum([u8; 20]),
    Substrate([u8; 32]),
}

impl From<[u8; 32]> for UniversalWalletAddress {
    fn from(v: [u8; 32]) -> Self {
        Self::Substrate(v)
    }
}

impl From<[u8; 20]> for UniversalWalletAddress {
    fn from(v: [u8; 20]) -> Self {
        Self::Ethereum(v)
    }
}

impl UniversalWalletAddress {
    /// Returns `true` if the universal wallet address is [`Unknown`].
    ///
    /// [`Unknown`]: UniversalWalletAddress::Unknown
    #[must_use]
    pub fn is_unknown(&self) -> bool {
        matches!(self, Self::Unknown)
    }

    /// Returns `true` if the universal wallet address is [`Ethereum`].
    ///
    /// [`Ethereum`]: UniversalWalletAddress::Ethereum
    #[must_use]
    pub fn is_ethereum(&self) -> bool {
        matches!(self, Self::Ethereum(..))
    }

    /// Returns `true` if the universal wallet address is [`Substrate`].
    ///
    /// [`Substrate`]: UniversalWalletAddress::Substrate
    #[must_use]
    pub fn is_substrate(&self) -> bool {
        matches!(self, Self::Substrate(..))
    }

    pub fn as_ethereum(&self) -> Option<Address> {
        if let Self::Ethereum(v) = self {
            Some(Address::from(*v))
        } else {
            None
        }
    }

    pub fn as_substrate(&self) -> Option<AccountId32> {
        if let Self::Substrate(v) = self {
            Some(AccountId32::from(*v))
        } else {
            None
        }
    }
}

impl core::fmt::Display for UniversalWalletAddress {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Unknown => write!(f, "Unknown"),
            Self::Ethereum(v) => write!(f, "{:?}", Address::from(*v)),
            Self::Substrate(v) => {
                write!(f, "{}", AccountId32::from(*v).to_ss58check())
            }
        }
    }
}

#[derive(
    Clone, Debug, Eq, Hash, PartialEq, serde::Serialize, serde::Deserialize,
)]
#[serde(tag = "type", content = "value")]
pub enum UserInfo {
    Twitter {
        id: u64,
        handle: String,
        address: UniversalWalletAddress,
    },
}

impl UserInfo {
    pub fn id(&self) -> u64 {
        match self {
            Self::Twitter { id, .. } => *id,
        }
    }

    pub fn name(&self) -> String {
        match self {
            Self::Twitter { handle, .. } => handle.clone(),
        }
    }

    pub fn address(&self) -> UniversalWalletAddress {
        match self {
            Self::Twitter { address, .. } => *address,
        }
    }
}

#[derive(
    Clone, Debug, Eq, Hash, PartialEq, serde::Serialize, serde::Deserialize,
)]
pub struct ClaimsData {
    pub identity: u64,
    pub address: UniversalWalletAddress,
    pub last_claimed_date: DateTime<Utc>,
}

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Invalid Access: {0}")]
    InvalidAccess(String),
    #[error("Invalid provider: {0}")]
    InvalidProvider(String),
}
