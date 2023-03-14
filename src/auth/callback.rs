use crate::error::Error;

use super::{
    super::{AppConfig, Auth, SledAuthorizer},
    get_token_cookie_name,
};
use rocket::{
    http::{Cookie, CookieJar, SameSite},
    response::Redirect,
    State,
};
use webb_auth::model::Provider;

fn make_cookie(name: &'static str, value: String, domain: &Option<String>) -> Cookie<'static> {
    let cookie = Cookie::build(name, value).same_site(SameSite::Lax);

    let cookie = match domain {
        Some(domain) => cookie.domain(domain.to_string()),
        None => cookie,
    };

    cookie.finish()
}

#[derive(FromForm, Debug)]
pub struct TwitterTokenResponse<'r> {
    oauth_token: &'r str,
    oauth_verifier: &'r str,
}

#[get("/auth/twitter?<token_response..>")]
pub async fn twitter(
    token_response: TwitterTokenResponse<'_>,
    cookies: &CookieJar<'_>,
    app_config: &State<AppConfig>,
    authorizer: &State<SledAuthorizer>,
    mut connection: &State<sled::Db>,
) -> Result<Redirect, Error> {
    if let Some(token) = authorizer
        .save_twitter_token(
            &mut connection,
            token_response.oauth_token,
            token_response.oauth_verifier,
        )
        .await?
    {
        cookies.add_private(make_cookie(
            get_token_cookie_name(Provider::Twitter),
            token,
            &app_config.domain,
        ));
    }

    let redirect = Redirect::to(app_config.default_login_redirect_uri.clone());

    Ok(redirect)
}
