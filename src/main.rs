#[macro_use]
extern crate rocket;

use rocket::{
    fairing::{AdHoc, Fairing, Info, Kind},
    http::Header,
    Build, Request, Response, Rocket,
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
pub mod faucet;

type SledAuthorizer = Authorizer<SledAuthDb>;

fn provider_fairing<P: IsProvider>() -> impl Fairing {
    OAuth2::<P>::fairing(P::provider().name())
}

pub struct CORS;

#[rocket::async_trait]
impl Fairing for CORS {
    fn info(&self) -> Info {
        Info {
            name: "Add CORS headers to responses",
            kind: Kind::Response,
        }
    }

    async fn on_response<'r>(&self, _request: &'r Request<'_>, response: &mut Response<'r>) {
        response.set_header(Header::new("Access-Control-Allow-Origin", "*"));
        response.set_header(Header::new(
            "Access-Control-Allow-Methods",
            "POST, GET, PATCH, OPTIONS",
        ));
        response.set_header(Header::new("Access-Control-Allow-Headers", "*"));
        response.set_header(Header::new("Access-Control-Allow-Credentials", "true"));
    }
}

#[derive(Deserialize)]
pub struct AppConfig {
    db: String,
    domain: Option<String>,
    default_login_redirect_uri: rocket::http::uri::Reference<'static>,
}

async fn init_authorization(rocket: &Rocket<Build>) -> Option<SledAuthorizer> {
    let twitter_config = OAuthConfig::from_figment(rocket.figment(), "twitter").ok()?;

    Authorizer::open(
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
        .mount(
            "/",
            routes![
                auth::login::status,
                auth::login::logout,
                auth::login::twitter,
                auth::callback::twitter,
                faucet::faucet,
            ],
        )
        .attach(provider_fairing::<Twitter>())
        .attach(CORS)
}
