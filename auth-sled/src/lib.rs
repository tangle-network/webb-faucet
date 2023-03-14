use egg_mode::{KeyPair, Token};
use rocket::State;
use std::convert::TryFrom;
use webb_auth::{AuthDb, UserInfo};

/// SledStore is a store that stores the history of events in  a [Sled](https://sled.rs)-based database.
#[derive(Clone)]
pub struct SledAuthDb;

#[async_trait::async_trait]
impl AuthDb for SledAuthDb {
    type Connection = State<sled::Db>;
    type Error = Error;

    async fn get_twitter_name(
        connection: &Self::Connection,
        id: u64,
    ) -> Result<Option<UserInfo>, Self::Error> {
        let id = u64_to_i64(id)?;
        let user_tree = connection.open_tree("users").unwrap();
        user_tree
            .get(&id.to_be_bytes())
            .map_err(Error::from)
            .map(|value| {
                value.map(|value| {
                    let user_object: UserInfo = serde_json::from_slice(&value).unwrap();
                    user_object
                })
            })
    }

    async fn put_twitter_name(
        connection: &Self::Connection,
        id: u64,
        value: &str,
    ) -> Result<(), Self::Error> {
        let id = u64_to_i64(id)?;
        let user_tree = connection.open_tree("users").unwrap();
        let user = UserInfo::Twitter {
            id: id.try_into().unwrap(),
            screen_name: value.to_string(),
            address: vec![],
        };
        let user_bytes = bincode::serialize(&user).unwrap();
        user_tree
            .insert(&id.to_be_bytes(), user_bytes)
            .map_err(Error::from)?;
        Ok(())
    }

    async fn put_twitter_name_with_address(
        connection: &Self::Connection,
        id: u64,
        value: &str,
        address: Vec<u8>,
    ) -> Result<(), Self::Error> {
        let id = u64_to_i64(id)?;
        let user_tree = connection.open_tree("users").unwrap();
        let user = UserInfo::Twitter {
            id: id.try_into().unwrap(),
            screen_name: value.to_string(),
            address,
        };
        let user_bytes = bincode::serialize(&user).unwrap();
        user_tree
            .insert(&id.to_be_bytes(), user_bytes)
            .map_err(Error::from)?;
        Ok(())
    }

    async fn lookup_twitter_token(
        connection: &Self::Connection,
        token: &str,
    ) -> Result<Option<u64>, Self::Error> {
        let token_tree = connection.open_tree("access_tokens").unwrap();
        token_tree.get(token).map_err(Error::from).map(|row| {
            row.map(|row| {
                let token: AccessToken = bincode::deserialize(&row).unwrap();
                token.id
            })
        })
    }

    async fn get_twitter_access_token(
        connection: &Self::Connection,
        token: &str,
    ) -> Result<Option<Token>, Self::Error> {
        let access_token_tree = connection.open_tree("access_tokens").unwrap();
        access_token_tree
            .get(token)
            .map_err(Error::from)
            .map(|row| {
                row.map(|row| {
                    let token: AccessToken = bincode::deserialize(&row).unwrap();
                    Token::Access {
                        consumer: KeyPair::new(token.token.to_string(), token.consumer_secret),
                        access: KeyPair::new(token.access_key, token.access_secret),
                    }
                })
            })
    }

    async fn put_twitter_token(
        connection: &Self::Connection,
        token: &str,
        id: u64,
        consumer_secret: &str,
        access_key: &str,
        access_secret: &str,
    ) -> Result<(), Self::Error> {
        let id = u64_to_i64(id)?;
        let access_token_tree = connection.open_tree("access_tokens").unwrap();
        let access_token = AccessToken {
            id: id.try_into().unwrap(),
            token: token.to_string(),
            consumer_secret: consumer_secret.to_string(),
            access_key: access_key.to_string(),
            access_secret: access_secret.to_string(),
        };
        let access_token_bytes = bincode::serialize(&access_token).unwrap();
        access_token_tree
            .insert(token, access_token_bytes)
            .map_err(Error::from)?;
        Ok(())
    }
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct AccessToken {
    token: String,
    id: u64,
    consumer_secret: String,
    access_key: String,
    access_secret: String,
}

fn u64_to_i64(value: u64) -> Result<i64, Error> {
    i64::try_from(value).map_err(|_| Error::InvalidId(value))
}

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Sled error")]
    Sled(#[from] sled::Error),
    #[error("Invalid ID")]
    InvalidId(u64),
}
