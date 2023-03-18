use model::Authorization;
use std::marker::PhantomData;

pub mod db;
pub mod model;

pub use db::AuthDb;
pub use model::{Provider, UserInfo};

pub struct Authorizer<A: AuthDb> {
    _auth_db: PhantomData<A>,
}

impl<A: AuthDb> Authorizer<A> {
    pub async fn open() -> Result<Self, Error<A::Error>> {
        Ok(Self {
            _auth_db: PhantomData,
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
    #[error("Auth DB error")]
    AuthDb(E),
}
