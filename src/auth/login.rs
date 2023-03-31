use rocket::http::CookieJar;
use rocket::response::Redirect;
use rocket_oauth2::OAuth2;
use twitter_v2::authorization::Scope;

use crate::auth::providers::Twitter;
use crate::error::Error;

#[get("/login/twitter")]
pub async fn twitter(oauth2: OAuth2<Twitter>, cookies: &CookieJar<'_>) -> Result<Redirect, Error> {
    Ok(oauth2.get_redirect(
        cookies,
        &[
            Scope::FollowsRead.to_string().as_str(),
            Scope::UsersRead.to_string().as_str(),
            Scope::OfflineAccess.to_string().as_str(),
            Scope::TweetRead.to_string().as_str(),
        ],
    )?)
}
