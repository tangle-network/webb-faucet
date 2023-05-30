#[macro_use]
extern crate rocket;

use std::{collections::HashMap, path::PathBuf};

use error::Error;
use ethers::{prelude::MiddlewareBuilder, signers::Signer};
use helpers::files::{get_evm_rpc_url, get_substrate_rpc_url};
use rocket::{
    fairing::{AdHoc, Fairing},
    futures::{stream::FuturesUnordered, StreamExt},
    http::Method,
};
use rocket::{launch, routes};
use rocket_cors::{AllowedOrigins, CorsOptions};
use rocket_oauth2::OAuth2;
use serde::Deserialize;
use sp_core::Pair;
use txes::{
    networks::Network,
    types::{EvmProviders, SubstrateProviders},
};
use webb::{
    evm::ethers::{
        prelude::{
            gas_escalator::{Frequency, GasEscalatorMiddleware, GeometricGasPrice},
            gas_oracle::{GasNow, GasOracleMiddleware},
            NonceManagerMiddleware, SignerMiddleware,
        },
        providers::{Http, Provider},
        signers::{coins_bip39::English, MnemonicBuilder},
    },
    substrate::subxt::{tx::PairSigner, OnlineClient, PolkadotConfig},
};

use webb_auth_sled::SledAuthDb;

pub mod auth;
pub mod error;
pub mod faucet;
pub mod helpers;
pub mod txes;

fn provider_fairing<P: auth::providers::Provider + 'static>() -> impl Fairing {
    OAuth2::<P>::fairing(P::name())
}

#[derive(Deserialize)]
pub struct AppConfig {
    db: PathBuf,
    mnemonic: String,
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

fn ethers_providers_firing() -> impl Fairing {
    AdHoc::on_ignite("Open provider", |rocket| async {
        let result = match rocket.state::<AppConfig>() {
            Some(config) => {
                let networks = vec![Network::Athena, Network::Hermes, Network::Demeter];

                let mnemonic = config.mnemonic.parse().unwrap();
                let wallet = match MnemonicBuilder::<English>::default()
                    .phrase(mnemonic)
                    .build()
                {
                    Ok(wallet) => wallet,
                    Err(_) => return Err(rocket),
                };
                rocket.manage(wallet);

                let providers: Vec<_> = networks
                    .iter()
                    .map(|net| net.to_evm_chain_id().unwrap())
                    .map(|chain_id| (chain_id, get_evm_rpc_url(chain_id)))
                    .map(|(chain_id, url)| {
                        let provider = Provider::<Http>::try_from(url)
                            .unwrap()
                            .wrap_into(|p| {
                                let escalator = GeometricGasPrice::new(1.125, 60_u64, None::<u64>);
                                GasEscalatorMiddleware::new(p, escalator, Frequency::PerBlock)
                            })
                            .wrap_into(|p| SignerMiddleware::new(p, wallet))
                            .wrap_into(|p| GasOracleMiddleware::new(p, GasNow::new()))
                            .wrap_into(|p| NonceManagerMiddleware::new(p, wallet.address()));
                        (chain_id, provider)
                    })
                    .into_iter()
                    .collect();

                let provider_map: HashMap<u64, _> = HashMap::new();
                for (chain_id, provider) in providers {
                    provider_map.insert(chain_id, provider);
                }
                Ok(provider_map)
            }
            None => Err(rocket),
        };

        match result {
            Ok(provider_map) => Ok(rocket.manage(EvmProviders {
                providers: provider_map,
            })),
            Err(_) => Err(rocket),
        }
    })
}

fn substrate_providers_firing() -> impl Fairing {
    AdHoc::on_ignite("Open provider", |rocket| async {
        let result = match rocket.state::<AppConfig>() {
            Some(config) => {
                let networks = vec![Network::Tangle];

                let mnemonic = config.mnemonic;
                let from = match sp_core::sr25519::Pair::from_string(&mnemonic, None) {
                    Ok(pair) => PairSigner::new(pair),
                    Err(_) => return Err(rocket),
                };
                rocket.manage(from);

                let mut futures_unordered = FuturesUnordered::new();

                for network in &networks {
                    let chain_id = network.to_substrate_chain_id().unwrap();
                    let url = get_substrate_rpc_url(chain_id);
                    futures_unordered.push(async move {
                        let api = OnlineClient::<PolkadotConfig>::from_url(url)
                            .await
                            .map_err(|e| Error::Custom(e.to_string()))?;
                        Ok::<_, Error>((chain_id, api))
                    });
                }

                let mut provider_map: HashMap<u64, OnlineClient<PolkadotConfig>> = HashMap::new();

                while let Some(result) = futures_unordered.next().await {
                    match result {
                        Ok((chain_id, api)) => {
                            provider_map.insert(chain_id, api);
                        }
                        Err(e) => {
                            println!("Error while opening provider: {}", e);
                            return Err(rocket);
                        }
                    }
                }

                Ok(provider_map)
            }
            None => return Err(rocket),
        };

        match result {
            Ok(provider_map) => Ok(rocket.manage(SubstrateProviders {
                providers: provider_map,
            })),
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
        .attach(ethers_providers_firing())
        .attach(cors.to_cors().unwrap())
        .manage(cors.to_cors().unwrap())
        .mount("/", rocket_cors::catch_all_options_routes())
        .mount("/", routes![auth::login::twitter, faucet::faucet,])
}
