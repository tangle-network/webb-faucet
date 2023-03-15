use egg_mode::Token;
use model::Authorization;
use std::marker::PhantomData;

use twitter::TwitterClient;

pub mod db;
pub mod model;
pub mod twitter;

pub use db::AuthDb;
pub use model::{Provider, UserInfo};

pub struct Authorizer<A: AuthDb> {
    _auth_db: PhantomData<A>,
    twitter_client: TwitterClient<A::Error>,
}

impl<A: AuthDb> Authorizer<A> {
    pub async fn open(
        twitter_client_id: &str,
        twitter_client_secret: &str,
        twitter_redirect_uri: &str,
    ) -> Result<Self, Error<A::Error>> {
        println!(
            "Setting up Authorizer with Twitter client ID: {}",
            twitter_client_id
        );
        Ok(Self {
            _auth_db: PhantomData,
            twitter_client: TwitterClient::new(
                twitter_client_id,
                twitter_client_secret,
                twitter_redirect_uri,
            ),
        })
    }

    pub async fn authorize_twitter(
        &self,
        connection: &A::Connection,
        token: &str,
    ) -> Result<Option<Authorization>, Error<A::Error>> {
        if let Some(id) = A::lookup_twitter_token(connection, token)
            .await
            .map_err(Error::AuthDb)?
        {
            A::save_authorization(connection, id)
                .await
                .map_err(Error::AuthDb)
        } else {
            Ok(None)
        }
    }

    pub async fn create_twitter_request_token(&self) -> Result<String, Error<A::Error>> {
        Ok(self.twitter_client.create_request_token().await?)
    }

    pub async fn save_twitter_token(
        &self,
        connection: &A::Connection,
        oauth_token: &str,
        oauth_verifier: &str,
    ) -> Result<Option<String>, Error<A::Error>> {
        match self
            .twitter_client
            .get_access_token(oauth_token, oauth_verifier)
            .await?
        {
            Some((Token::Access { consumer, access }, user_id, screen_name)) => {
                A::put_twitter_name(connection, user_id, &screen_name)
                    .await
                    .map_err(Error::AuthDb)?;
                A::put_twitter_token(
                    connection,
                    &consumer.key,
                    user_id,
                    &consumer.secret,
                    &access.key,
                    &access.secret,
                )
                .await
                .map_err(Error::AuthDb)?;

                Ok(Some(consumer.key.to_string()))
            }
            _ => Ok(None),
        }
    }

    pub async fn get_user_info(
        &self,
        connection: &A::Connection,
        id: &u64,
    ) -> Result<Option<UserInfo>, Error<A::Error>> {
        Ok(A::get_twitter_name(connection, *id)
            .await
            .map_err(Error::AuthDb)?)
    }
}

#[derive(thiserror::Error, Debug)]
pub enum Error<E: std::error::Error> {
    #[error("Twitter API error")]
    TwitterApi(#[from] egg_mode::error::Error),
    #[error("Auth DB error")]
    AuthDb(E),
}
