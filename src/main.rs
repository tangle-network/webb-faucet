#[macro_use]
extern crate rocket;

use std::path::PathBuf;

use rocket::{
    fairing::{AdHoc, Fairing},
    http::Method,
};
use rocket::{launch, routes};
use rocket_cors::{AllowedOrigins, CorsOptions};
use rocket_oauth2::OAuth2;
use serde::Deserialize;
use webb_auth_sled::SledAuthDb;

pub mod auth;
pub mod error;
pub mod faucet;

fn provider_fairing<P: auth::providers::Provider + 'static>() -> impl Fairing {
    OAuth2::<P>::fairing(P::name())
}

#[derive(Deserialize)]
pub struct AppConfig {
    db: PathBuf,
}

fn auth_db_firing() -> impl Fairing {
    AdHoc::try_on_ignite("Open Auth database", |rocket| async {
        let maybe_db = match rocket.state::<AppConfig>() {
            Some(config) => {
                println!("Opening database at {}", config.db.display());
                SledAuthDb::open(&config.db)
            }
            None => return Err(rocket),
        };
        match maybe_db {
            Ok(db) => Ok(rocket.manage(db)),
            Err(_) => Err(rocket),
        }
    })
}

#[launch]
async fn rocket() -> _ {
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
        .attach(auth_db_firing())
        .attach(provider_fairing::<auth::providers::Twitter>())
        .attach(cors.to_cors().unwrap())
        .manage(cors.to_cors().unwrap())
        .mount("/", rocket_cors::catch_all_options_routes())
        .mount("/", routes![auth::login::twitter, faucet::faucet,])
}
