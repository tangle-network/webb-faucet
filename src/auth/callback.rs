use crate::error::Error;

use super::super::AppConfig;
use rocket::{
    http::{Cookie, CookieJar, SameSite},
    response::Redirect,
    State,
};
use twitter_v2::authorization::BearerToken;
use twitter_v2::TwitterApi;
use webb_auth::AuthDb;
use webb_auth_sled::SledAuthDb;

fn make_cookie(name: &'static str, value: String, domain: &Option<String>) -> Cookie<'static> {
    let cookie = Cookie::build(name, value).same_site(SameSite::Lax);

    let cookie = match domain {
        Some(domain) => cookie.domain(domain.to_string()),
        None => cookie,
    };

    cookie.finish()
}

#[get("/auth/twitter?<access_token>")]
pub async fn twitter(
    access_token: String,
    cookies: &CookieJar<'_>,
    app_config: &State<AppConfig>,
    connection: &State<sled::Db>,
) -> Result<Redirect, Error> {
    cookies.add_private(
        Cookie::build("token", access_token.clone())
            .same_site(SameSite::Lax)
            .finish(),
    );

    println!("access_token: {}", access_token);
    let auth = BearerToken::new(access_token);
    let twitter_api = TwitterApi::new(auth);

    let maybe_user: Option<twitter_v2::data::User> = twitter_api
        .get_users_me()
        .send()
        .await
        .map_err(|e| {
            println!("error: {:?}", e);
            Error::TwitterError(e)
        })
        .map(|res| match res.data.clone() {
            Some(data) => Some(data),
            None => None,
        })?;

    match maybe_user {
        Some(user) => {
            <SledAuthDb as AuthDb>::put_twitter_name(connection, user.id.into(), &user.username)
                .await
                .map_err(|e| {
                    println!("error: {:?}", e);
                    Error::Custom(format!("Error: {:?}", e.to_string()))
                })?;
            println!("Successfully put twitter name into the database");
        }
        None => {
            return Err(Error::TwitterError(twitter_v2::error::Error::Custom(
                "No user found".to_string(),
            )))
        }
    };

    let redirect = Redirect::to(app_config.default_login_redirect_uri.clone());
    Ok(redirect)
}
