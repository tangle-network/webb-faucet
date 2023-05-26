use serde::Deserialize;
use serde::Serialize;
use webb_auth::model::UniversalWalletAddress;

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(crate = "rocket::serde")]
#[serde(tag = "type", content = "value", rename_all = "camelCase")]
pub enum MultiAddress {
    Ethereum(webb::evm::ethers::types::Address),
    Substrate(webb::substrate::subxt::utils::AccountId32),
}

impl MultiAddress {
    /// Returns `true` if the multi address is [`Ethereum`].
    ///
    /// [`Ethereum`]: MultiAddress::Ethereum
    #[must_use]
    pub fn is_ethereum(&self) -> bool {
        matches!(self, Self::Ethereum(..))
    }

    /// Returns `true` if the multi address is [`Substrate`].
    ///
    /// [`Substrate`]: MultiAddress::Substrate
    #[must_use]
    pub fn is_substrate(&self) -> bool {
        matches!(self, Self::Substrate(..))
    }

    /// Returns the [`Ethereum`] address if the multi address is [`Ethereum`].
    /// Returns `None` if the multi address is [`Substrate`].
    #[must_use]
    pub fn ethereum(&self) -> Option<&webb::evm::ethers::types::Address> {
        match self {
            Self::Ethereum(address) => Some(address),
            Self::Substrate(..) => None,
        }
    }

    /// Returns the [`Substrate`] address if the multi address is [`Substrate`].
    /// Returns `None` if the multi address is [`Ethereum`].
    #[must_use]
    pub fn substrate(&self) -> Option<&webb::substrate::subxt::utils::AccountId32> {
        match self {
            Self::Ethereum(..) => None,
            Self::Substrate(address) => Some(address),
        }
    }
}

impl core::fmt::Display for MultiAddress {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Ethereum(address) => write!(f, "{}", address),
            Self::Substrate(address) => write!(f, "{}", address),
        }
    }
}

impl From<MultiAddress> for UniversalWalletAddress {
    fn from(multi_address: MultiAddress) -> Self {
        match multi_address {
            MultiAddress::Ethereum(address) => Self::Ethereum(address.to_fixed_bytes()),
            MultiAddress::Substrate(address) => Self::Substrate(address.0),
        }
    }
}
