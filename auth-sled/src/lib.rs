use chrono::{DateTime, Utc};
use std::convert::TryFrom;
use webb_auth::{model::ClaimsData, AuthDb, UserInfo};
use webb_proposals::TypedChainId;

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

    async fn put_user_info(&self, id: u64, value: &UserInfo) -> Result<(), Self::Error> {
        let id = u64_to_i64(id)?;
        let user_info_tree = self.db.open_tree("users")?;
        let user_info_bytes = serde_json::to_vec(value)?;
        user_info_tree.insert(id.to_be_bytes(), user_info_bytes)?;
        Ok(())
    }

    async fn get_user_info(&self, id: u64) -> Result<Option<UserInfo>, Self::Error> {
        let id = u64_to_i64(id)?;
        let user_info_tree = self.db.open_tree("users")?;
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
        let last_claim_tree = self
            .db
            .open_tree(format!("claims-{}", typed_chain_id.chain_id()))?;
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
        let last_claim_tree = self
            .db
            .open_tree(format!("claims-{}", typed_chain_id.chain_id()))?;
        last_claim_tree
            .get(id.to_be_bytes())
            .map_err(Into::into)
            .and_then(|row| {
                row.map(|row| serde_json::from_slice(&row).map_err(Into::into))
                    .transpose()
            })
    }
}

fn u64_to_i64(value: u64) -> Result<i64, Error> {
    i64::try_from(value).map_err(|_| Error::InvalidId(value))
}

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Sled error: {0}")]
    Sled(#[from] sled::Error),
    #[error("Invalid ID: {0}")]
    InvalidId(u64),
    #[error("Invalid Serialization: {0}")]
    Serde(#[from] serde_json::Error),
}
