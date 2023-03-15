use super::authorization::Error;
use flagset::{flags, FlagSet};
use std::fmt::Display;
use std::str::FromStr;

pub mod providers {
    use super::{IsProvider, Provider};

    pub struct Twitter;

    impl IsProvider for Twitter {
        type Id = u64;

        fn provider() -> Provider {
            Provider::Twitter
        }
    }
}

pub trait IsProvider: 'static {
    type Id;

    fn provider() -> Provider;
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Provider {
    Twitter,
}

impl Provider {
    pub const fn prefix(&self) -> &'static str {
        match self {
            Self::Twitter => "tw",
        }
    }

    pub const fn name(&self) -> &'static str {
        match self {
            Self::Twitter => "twitter",
        }
    }
}

impl Display for Provider {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name())
    }
}

impl FromStr for Provider {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_ref() {
            "twitter" => Ok(Self::Twitter),
            other => Err(Error::InvalidProvider(other.to_string())),
        }
    }
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum Identity {
    Twitter { id: u64 },
}

impl Identity {
    pub fn provider(&self) -> Provider {
        match self {
            Identity::Twitter { .. } => Provider::Twitter,
        }
    }

    pub fn for_provider(provider: Provider, id: &str, name: &str) -> Result<Identity, Error> {
        match provider {
            Provider::Twitter => {
                let id = id
                    .parse::<u64>()
                    .map_err(|_| Error::InvalidIdentifier(id.to_string()))?;
                Ok(Identity::Twitter { id })
            }
        }
    }
}

flags! {
    pub enum Access: u8 {
        Admin,
        Trusted,
    }
}

impl Access {
    pub const fn name(&self) -> &'static str {
        match self {
            Self::Admin => "admin",
            Self::Trusted => "trusted",
        }
    }

    pub fn from_label(s: &str) -> Result<FlagSet<Self>, Error> {
        match s {
            "admin" => Ok(Access::Admin | Access::Trusted),
            "trusted" => Ok(Access::Trusted.into()),
            other => Err(Error::InvalidAccess(other.to_string())),
        }
    }
}

impl Display for Access {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name())
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Authorization {
    pub identity: Identity,
    access: FlagSet<Access>,
}

impl Authorization {
    pub fn new<I: Into<FlagSet<Access>>>(identity: Identity, access: I) -> Self {
        Self {
            identity,
            access: access.into(),
        }
    }

    pub fn provider(&self) -> Provider {
        self.identity.provider()
    }

    pub fn is_admin(&self) -> bool {
        self.access.contains(Access::Admin)
    }

    pub fn is_trusted(&self) -> bool {
        self.access.contains(Access::Trusted)
    }
}

#[derive(Clone, Debug, Eq, Hash, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum UserInfo {
    Twitter {
        id: u64,
        screen_name: String,
        address: Vec<u8>,
    },
}

impl UserInfo {
    pub fn id_str(&self) -> String {
        match self {
            Self::Twitter { id, .. } => id.to_string(),
        }
    }

    pub fn name(&self) -> String {
        match self {
            Self::Twitter { screen_name, .. } => screen_name.to_string(),
        }
    }

    pub fn address(&self) -> Vec<u8> {
        match self {
            Self::Twitter { address, .. } => address.clone(),
        }
    }
}
