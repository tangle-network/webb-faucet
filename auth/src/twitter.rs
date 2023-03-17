use chrono::{DateTime, Duration, Utc};
use egg_mode::{auth::Token, KeyPair};
use parking_lot::RwLock;
use std::{collections::HashMap, marker::PhantomData};

use crate::Error;

pub struct TwitterClient<E: std::error::Error> {
    consumer_token: KeyPair,
    redirect_uri: String,
    request_tokens: RwLock<HashMap<String, (String, DateTime<Utc>)>>,
    phantom: PhantomData<E>,
}

impl<E: std::error::Error> TwitterClient<E> {
    pub fn new(client_id: &str, client_secret: &str, redirect_uri: &str) -> Self {
        Self {
            consumer_token: KeyPair::new(client_id.to_string(), client_secret.to_string()),
            redirect_uri: redirect_uri.to_string(),
            request_tokens: RwLock::new(HashMap::new()),
            phantom: PhantomData,
        }
    }

    pub fn expire(&self, max_age: Duration) {
        let now = Utc::now();

        self.request_tokens
            .write()
            .retain(|_, (_, created)| (now - *created) < max_age)
    }

    pub async fn create_request_token(&self) -> Result<String, Error<E>> {
        let request_token = egg_mode::auth::request_token(&self.consumer_token, &self.redirect_uri)
            .await
            .map_err(|e| {
                println!("Error creating request token: {:?}", e);
                Error::TwitterApi(e)
            })?;
        self.put_request_token(&request_token.key, &request_token.secret);
        Ok(request_token.key.to_string())
    }

    pub async fn get_access_token(
        &self,
        oauth_token: &str,
        oauth_verifier: &str,
    ) -> Result<Option<(Token, u64, String)>, Error<E>> {
        if let Some(secret) = self.get_secret(oauth_token) {
            Ok(egg_mode::auth::access_token(
                self.consumer_token.clone(),
                &egg_mode::KeyPair::new(oauth_token.to_string(), secret),
                oauth_verifier,
            )
            .await
            .map(Some)?)
        } else {
            Ok(None)
        }
    }

    fn put_request_token(&self, key: &str, secret: &str) {
        let key = key.to_string();
        let secret = secret.to_string();

        self.request_tokens
            .write()
            .insert(key, (secret, Utc::now()));
    }

    fn get_secret(&self, key: &str) -> Option<String> {
        self.request_tokens
            .write()
            .remove(key)
            .map(|(secret, _)| secret)
    }
}
