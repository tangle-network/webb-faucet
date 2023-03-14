use egg_mode::Token;

use crate::UserInfo;

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
}
