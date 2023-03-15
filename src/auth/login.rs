use super::super::{AppConfig, SledAuthorizer};
use crate::error::Error;
use rocket::{
    http::{hyper::body::HttpBody, CookieJar},
    response::Redirect,
    State,
};
use rocket_oauth2::OAuth2;
use serde::Serialize;
use serde_json::Value;
use webb_auth::model::{providers::Twitter, Access, Authorization, Provider, UserInfo};

#[derive(Default, Serialize)]
struct LoginStatus {
    twitter: Option<ProviderStatus>,
}

#[derive(Serialize)]
struct ProviderStatus {
    id: String,
    name: String,
    access: Access,
}

impl ProviderStatus {
    fn new(authorization: &Authorization, user_info: &UserInfo) -> Self {
        Self {
            id: user_info.id_str(),
            name: user_info.name(),
            access: authorization.access,
        }
    }
}

#[get("/login/status")]
pub async fn status(
    cookies: &CookieJar<'_>,
    authorizer: &State<SledAuthorizer>,
    connection: &State<sled::Db>,
) -> Result<Value, Error> {
    let mut status = LoginStatus::default();
    if let Some(token) = super::get_token_cookie(cookies, Provider::Twitter) {
        if let Some(authorization) = authorizer.authorize_twitter(connection, &token).await? {
            if let Some(user_info) = authorizer
                .get_user_info(connection, &authorization.identity)
                .await?
            {
                status.twitter = Some(ProviderStatus::new(&authorization, &user_info));
            }
        }
    }

    Ok(serde_json::json!(status))
}

#[get("/logout")]
pub fn logout(cookies: &CookieJar<'_>, app_config: &State<AppConfig>) -> Redirect {
    for token_cookie_name in super::TOKEN_COOKIE_NAMES {
        if let Some(mut cookie) = cookies.get_private(token_cookie_name) {
            if let Some(domain) = &app_config.domain {
                cookie.set_domain(domain.clone());
            }

            cookies.remove_private(cookie);
        }
    }

    Redirect::to(app_config.default_login_redirect_uri.clone())
}

#[get("/login/twitter")]
pub async fn twitter(
    oauth2: OAuth2<Twitter>,
    cookies: &CookieJar<'_>,
    authorizer: &State<SledAuthorizer>,
) -> Result<Redirect, Error> {
    let request_token_key = authorizer
        .create_twitter_request_token()
        .await
        .map_err(|e| {
            println!("Error creating twitter request token: {}", e);
            Error::Authorization(e)
        })?;
    println!("Creating twitter request token {}", request_token_key);
    Ok(oauth2.get_redirect_extras(cookies, &[], &[("oauth_token", &request_token_key)])?)
}
