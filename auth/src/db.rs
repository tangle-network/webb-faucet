use chrono::{DateTime, Utc};
use webb_proposals::TypedChainId;

use crate::{model::ClaimsData, UserInfo};

#[async_trait::async_trait]
pub trait AuthDb {
    type Error: std::error::Error;

    async fn get_user_info(&self, id: u64) -> Result<Option<UserInfo>, Self::Error>;

    async fn put_user_info(&self, id: u64, value: &UserInfo) -> Result<(), Self::Error>;

    async fn put_last_claim_data(
        &self,
        id: u64,
        typed_chain_id: TypedChainId,
        claim: ClaimsData,
    ) -> Result<DateTime<Utc>, Self::Error>;

    async fn get_last_claim_data(
        &self,
        id: u64,
        typed_chain_id: TypedChainId,
    ) -> Result<Option<ClaimsData>, Self::Error>;
}
