use chrono::{DateTime, Utc};
use egg_mode::Token;
use webb_proposals::TypedChainId;

use crate::{
    model::{Authorization, ClaimsData},
    UserInfo,
};

#[async_trait::async_trait]
pub trait AuthDb {
    type Connection;
    type Error: std::error::Error;

    async fn get_twitter_name(
        connection: &Self::Connection,
        id: u64,
    ) -> Result<Option<UserInfo>, Self::Error>;

    async fn put_twitter_name(
        connection: &Self::Connection,
        id: u64,
        value: &str,
    ) -> Result<(), Self::Error>;

    async fn put_twitter_name_with_address(
        connection: &Self::Connection,
        id: u64,
        value: &str,
        address: Vec<u8>,
    ) -> Result<(), Self::Error>;

    async fn lookup_twitter_token(
        connection: &Self::Connection,
        token: &str,
    ) -> Result<Option<u64>, Self::Error>;

    async fn get_twitter_access_token(
        connection: &Self::Connection,
        token: &str,
    ) -> Result<Option<Token>, Self::Error>;

    async fn put_twitter_token(
        connection: &Self::Connection,
        token: &str,
        id: u64,
        consumer_secret: &str,
        access_key: &str,
        access_secret: &str,
    ) -> Result<(), Self::Error>;

    async fn save_authorization(
        connection: &Self::Connection,
        id: u64,
    ) -> Result<Option<Authorization>, Self::Error>;

    async fn put_last_claim_date(
        connection: &Self::Connection,
        id: u64,
        typed_chain_id: TypedChainId,
        claim: ClaimsData,
    ) -> Result<DateTime<Utc>, Self::Error>;

    async fn get_last_claim_date(
        connection: &Self::Connection,
        id: u64,
        typed_chain_id: TypedChainId,
    ) -> Result<Option<DateTime<Utc>>, Self::Error>;
}
