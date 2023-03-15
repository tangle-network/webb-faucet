use std::fmt::Display;
use std::str::FromStr;

use chrono::{DateTime, Utc};

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

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum Access {
    Admin,
    Trusted,
    Untrusted,
}

impl Access {
    pub const fn name(&self) -> &'static str {
        match self {
            Self::Admin => "admin",
            Self::Trusted => "trusted",
            Self::Untrusted => "untrusted",
        }
    }

    pub fn from_label(s: &str) -> Result<Access, Error> {
        match s {
            "admin" => Ok(Access::Admin),
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

#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Authorization {
    pub identity: u64,
    pub access: Access,
    pub has_claimed_date: Option<DateTime<Utc>>,
}

impl Authorization {
    pub fn new(identity: u64, access: Access) -> Self {
        Self {
            identity,
            access: access.into(),
            has_claimed_date: None,
        }
    }

    pub fn update_claimed_date(&mut self, date: DateTime<Utc>) {
        self.has_claimed_date = Some(date);
    }

    pub fn is_claim_older_than_one_day(&self) -> bool {
        match self.has_claimed_date {
            Some(date) => date < Utc::now() - chrono::Duration::days(1),
            None => true,
        }
    }

    pub fn provider(&self) -> Provider {
        Provider::Twitter
    }

    pub fn is_admin(&self) -> bool {
        match self.access {
            Access::Admin => true,
            _ => false,
        }
    }

    pub fn is_trusted(&self) -> bool {
        match self.access {
            Access::Admin | Access::Trusted => true,
            _ => false,
        }
    }

    pub fn date_updated(&self) -> Option<DateTime<Utc>> {
        self.has_claimed_date
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

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Invalid Access")]
    InvalidAccess(String),
    #[error("Invalid provider")]
    InvalidProvider(String),
}
