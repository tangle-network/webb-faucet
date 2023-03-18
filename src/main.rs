#[macro_use]
extern crate rocket;

use rocket::{
    fairing::{AdHoc, Fairing, Info, Kind},
    http::{Header, Method},
    Build, Request, Response, Rocket,
};
use rocket::{launch, routes};
use rocket_cors::{AllowedOrigins, CorsOptions};
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

#[derive(Deserialize)]
pub struct AppConfig {
    db: String,
    domain: Option<String>,
    default_login_redirect_uri: rocket::http::uri::Reference<'static>,
}

async fn init_authorization(rocket: &Rocket<Build>) -> Option<SledAuthorizer> {
    let _twitter_config = OAuthConfig::from_figment(rocket.figment(), "twitter").ok()?;

    Authorizer::open()
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
    let cors = CorsOptions::default()
        .allowed_origins(AllowedOrigins::all())
        .allowed_methods(
            vec![Method::Get, Method::Post, Method::Patch]
                .into_iter()
                .map(From::from)
                .collect(),
        )
        .allow_credentials(true);

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
        .attach(cors.to_cors().unwrap())
        .manage(cors.to_cors().unwrap())
        .mount("/", rocket_cors::catch_all_options_routes())
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
}
