use chrono::{DateTime, Utc};
use std::convert::TryFrom;
use webb_auth::AuthDb;
use webb_proposals::TypedChainId;

pub use webb_auth::model::*;
/// SledStore is a store that stores the history of events in  a [Sled](https://sled.rs)-based database.
#[derive(Clone)]
pub struct SledAuthDb {
    db: sled::Db,
}

impl SledAuthDb {
    /// Open a new SledStore.
    pub fn open<P: AsRef<std::path::Path>>(path: P) -> Result<Self, Error> {
        Ok(Self {
            db: sled::Config::new()
                .use_compression(false)
                .path(path)
                .open()?,
        })
    }

    pub fn user_info_tree(&self) -> Result<sled::Tree, Error> {
        self.db.open_tree("users").map_err(Into::into)
    }

    pub fn claims_tree(
        &self,
        chain_id: TypedChainId,
    ) -> Result<sled::Tree, Error> {
        self.db
            .open_tree(format!("claims-{}", chain_id.chain_id()))
            .map_err(Into::into)
    }

    /// Open a new SledStore in a temporary directory.
    #[cfg(test)]
    pub fn open_for_tests() -> Result<Self, Error> {
        Ok(Self {
            db: sled::Config::new().temporary(true).open()?,
        })
    }
}

#[async_trait::async_trait]
impl AuthDb for SledAuthDb {
    type Error = Error;

    async fn put_user_info(
        &self,
        id: u64,
        value: &UserInfo,
    ) -> Result<(), Self::Error> {
        let id = u64_to_i64(id)?;
        let user_info_tree = self.user_info_tree()?;
        let user_info_bytes = serde_json::to_vec(value)?;
        user_info_tree.insert(id.to_be_bytes(), user_info_bytes)?;
        Ok(())
    }

    async fn get_user_info(
        &self,
        id: u64,
    ) -> Result<Option<UserInfo>, Self::Error> {
        let id = u64_to_i64(id)?;
        let user_info_tree = self.user_info_tree()?;
        user_info_tree
            .get(id.to_be_bytes())
            .map_err(Into::into)
            .and_then(|row| {
                row.map(|row| serde_json::from_slice(&row).map_err(Into::into))
                    .transpose()
            })
    }

    async fn put_last_claim_data(
        &self,
        id: u64,
        typed_chain_id: TypedChainId,
        claim: ClaimsData,
    ) -> Result<DateTime<Utc>, Self::Error> {
        let id = u64_to_i64(id)?;
        let last_claim_tree = self.claims_tree(typed_chain_id)?;
        let claims_data_bytes = serde_json::to_vec(&claim)?;
        last_claim_tree.insert(id.to_be_bytes(), claims_data_bytes)?;
        Ok(claim.last_claimed_date)
    }

    async fn get_last_claim_data(
        &self,
        id: u64,
        typed_chain_id: TypedChainId,
    ) -> Result<Option<ClaimsData>, Self::Error> {
        let id = u64_to_i64(id)?;
        let last_claim_tree = self.claims_tree(typed_chain_id)?;
        last_claim_tree
            .get(id.to_be_bytes())
            .map_err(Into::into)
            .and_then(|row| {
                row.map(|row| serde_json::from_slice(&row).map_err(Into::into))
                    .transpose()
            })
    }
}

pub fn u64_to_i64(value: u64) -> Result<i64, Error> {
    i64::try_from(value).map_err(|_| Error::InvalidU65Id(value))
}

pub fn i64_to_u64(value: i64) -> Result<u64, Error> {
    u64::try_from(value).map_err(|_| Error::InvalidI65Id(value))
}

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Sled error: {0}")]
    Sled(#[from] sled::Error),
    #[error("Invalid ID: {0}")]
    InvalidU65Id(u64),
    #[error("Invalid ID: {0}")]
    InvalidI65Id(i64),
    #[error("Invalid Serialization: {0}")]
    Serde(#[from] serde_json::Error),
}
