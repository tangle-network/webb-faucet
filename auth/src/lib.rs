use egg_mode::Token;
use std::marker::PhantomData;
use std::path::Path;

use authorization::Authorizations;
use twitter::TwitterClient;

mod authorization;
pub mod db;
pub mod model;
pub mod twitter;

pub use db::AuthDb;
pub use model::{Access, Authorization, Identity, Provider, UserInfo};

pub struct Authorizer<A> {
    _auth_db: PhantomData<A>,
    authorizations: Authorizations,
    twitter_client: TwitterClient,
}

impl<A: AuthDb> Authorizer<A> {
    pub async fn open<P: AsRef<Path>>(
        authorizations_path: P,
        twitter_client_id: &str,
        twitter_client_secret: &str,
        twitter_redirect_uri: &str,
    ) -> Result<Self, Error<A::Error>> {
        println!("Opening authorizations file: {:?}", authorizations_path.as_ref());
        Ok(Self {
            _auth_db: PhantomData,
            authorizations: Authorizations::open(authorizations_path)?,
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
            let identity = Identity::Twitter { id };
            let access = self.authorizations.lookup(&identity);

            Ok(Some(Authorization::new(identity, access)))
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
        identity: &Identity,
    ) -> Result<Option<UserInfo>, Error<A::Error>> {
        Ok(match identity {
            Identity::Twitter { id } => A::get_twitter_name(connection, *id)
                .await
                .map_err(Error::AuthDb)?,
        })
    }
}

#[derive(thiserror::Error, Debug)]
pub enum Error<E: std::error::Error> {
    #[error("Twitter client error")]
    TwitterClient(#[from] twitter::Error),
    #[error("Authorizations file error")]
    Authorizations(#[from] authorization::Error),
    #[error("Auth DB error")]
    AuthDb(E),
}
