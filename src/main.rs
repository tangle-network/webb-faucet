#[macro_use]
extern crate rocket;

use rocket::{
    fairing::{AdHoc, Fairing},
    Build, Rocket,
};
use rocket::{launch, routes};
use rocket_oauth2::{OAuth2, OAuthConfig};
use serde::Deserialize;
use webb_auth::{
    model::{providers::Twitter, IsProvider},
    Authorizer,
};
use webb_auth_sled::SledAuthDb;

pub mod auth;
pub mod error;

type SledAuthorizer = Authorizer<SledAuthDb>;
pub struct Auth(sled::Db);

fn provider_fairing<P: IsProvider>() -> impl Fairing {
    OAuth2::<P>::fairing(P::provider().name())
}

#[derive(Deserialize)]
pub struct AppConfig {
    db: String,
    authorization: String,
    domain: Option<String>,
    default_login_redirect_uri: rocket::http::uri::Reference<'static>,
}

async fn init_authorization(rocket: &Rocket<Build>) -> Option<SledAuthorizer> {
    let twitter_config = OAuthConfig::from_figment(rocket.figment(), "twitter").ok()?;
    let config = rocket.state::<AppConfig>()?;

    Authorizer::open(
        &config.authorization,
        twitter_config.client_id(),
        twitter_config.client_secret(),
        twitter_config.redirect_uri()?,
    )
    .await
    .ok()
}

fn init_db(rocket: &Rocket<Build>) -> Option<sled::Db> {
    let config = rocket.state::<AppConfig>()?;
    let db: sled::Db = sled::open(config.db.clone()).unwrap();
    Some(db)
}

#[launch]
fn rocket() -> _ {
    rocket::build()
        .attach(AdHoc::config::<AppConfig>())
        .attach(AdHoc::try_on_ignite("Open database", |rocket| async {
            match init_db(&rocket) {
                Some(db) => Ok(rocket.manage(db)),
                None => Err(rocket),
            }
        }))
        .attach(AdHoc::try_on_ignite(
            "Open authorization databases",
            |rocket| async {
                match init_authorization(&rocket).await {
                    Some(authorizer) => Ok(rocket.manage(authorizer)),
                    None => Err(rocket),
                }
            },
        ))
        .attach(provider_fairing::<Twitter>())
        .mount(
            "/",
            routes![
                auth::login::status,
                auth::login::logout,
                auth::login::twitter,
                auth::callback::twitter,
            ],
        )
}
