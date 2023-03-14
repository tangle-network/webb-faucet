use super::{error::Error, Auth, SledAuthorizer};
use rocket::{http::CookieJar, State};
use webb_auth::model::Provider;

pub mod callback;
pub mod login;

const TOKEN_COOKIE_NAMES: [&str; 1] = [get_token_cookie_name(Provider::Twitter)];

const fn get_token_cookie_name(provider: Provider) -> &'static str {
    match provider {
        Provider::Twitter => "twitter_token",
    }
}

pub fn get_token_cookie(cookies: &CookieJar<'_>, provider: Provider) -> Option<String> {
    cookies
        .get_private(get_token_cookie_name(provider))
        .map(|cookie| cookie.value().to_string())
}

pub async fn lookup_is_trusted(
    cookies: &CookieJar<'_>,
    authorizer: &SledAuthorizer,
    mut connection: State<sled::Db>,
) -> Result<bool, Error> {
    match get_token_cookie(cookies, Provider::Twitter) {
        Some(token) => authorizer
            .authorize_twitter(&mut connection, &token)
            .await?
            .map(|authorization| Ok(authorization.is_trusted()))
            .unwrap_or(Ok(false)),
        None => Ok(false),
    }
}
