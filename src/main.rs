#[macro_use]
extern crate rocket;

use std::sync::Arc;
use std::{collections::HashMap, path::PathBuf};

use error::Error;
use ethers::{
    prelude::MiddlewareBuilder, signers::Signer, types::PathOrString,
};
use helpers::files::{get_evm_rpc_url, get_substrate_rpc_url};
use rocket::tokio::sync::mpsc;
use rocket::{
    fairing::{AdHoc, Fairing},
    futures::{stream::FuturesUnordered, StreamExt},
    http::Method,
};
use rocket::{launch, log, routes};
use rocket_cors::{AllowedOrigins, CorsOptions};
use rocket_oauth2::OAuth2;
use serde::Deserialize;
use sp_core::Pair;
use txes::{
    networks::Network,
    processor::TransactionProcessingSystem,
    types::{EvmProviders, SubstrateProviders},
};
use webb::evm::ethers;
use webb::substrate::subxt;
use webb::{
    evm::ethers::{
        prelude::{
            gas_escalator::{
                Frequency, GasEscalatorMiddleware, GeometricGasPrice,
            },
            gas_oracle::GasNow,
        },
        providers::{Http, Provider},
        signers::{coins_bip39::English, MnemonicBuilder},
    },
    substrate::subxt::{OnlineClient, PolkadotConfig},
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

fn default_time_to_wait_between_claims() -> std::time::Duration {
    // check if the rust env is debug, if so, skip the 24 hour check
    let rocket_profile = std::env::var("ROCKET_PROFILE").unwrap_or_default();
    if rocket_profile == "debug" {
        std::time::Duration::from_secs(0)
    } else {
        std::time::Duration::from_secs(24 * 60 * 60)
    }
}

const fn default_token_amount() -> u64 {
    20
}

const fn default_native_token_amount() -> f64 {
    0.5
}

const fn default_verify_following_webb() -> bool {
    true
}

#[derive(Deserialize)]
pub struct AppConfig {
    /// The database to use for the auth and claims
    db: PathBuf,
    /// The mnemonic to use for the faucet
    mnemonic: String,
    /// The amount of time to wait between claims
    /// This is to prevent users from spamming the faucet
    #[serde(default = "default_time_to_wait_between_claims")]
    pub time_to_wait_between_claims: std::time::Duration,
    /// The amount of ERC20 tokens to send to the user
    #[serde(default = "default_token_amount")]
    pub token_amount: u64,
    /// The amount of native tokens to send to the user
    #[serde(default = "default_native_token_amount")]
    pub native_token_amount: f64,
    /// Whether to verify that the user is following the webb twitter account
    #[serde(default = "default_verify_following_webb")]
    pub verify_following_webb: bool,
}

fn auth_db_firing() -> impl Fairing {
    AdHoc::try_on_ignite("Open Auth database", |rocket| async {
        let maybe_db = match rocket.state::<AppConfig>() {
            Some(config) => SledAuthDb::open(&config.db),
            None => return Err(rocket),
        };
        match maybe_db {
            Ok(db) => Ok(rocket.manage(db)),
            Err(_) => Err(rocket),
        }
    })
}

fn ethers_wallet_firing() -> impl Fairing {
    AdHoc::try_on_ignite("Open ethers-rs wallet", |rocket| async {
        let maybe_wallet = match rocket.state::<AppConfig>() {
            Some(config) => {
                let mnemonic: String =
                    config.mnemonic.parse().expect("Mnemonic is not valid");
                MnemonicBuilder::<English>::default()
                    .phrase(PathOrString::String(mnemonic))
                    .build()
            }
            None => return Err(rocket),
        };

        match maybe_wallet {
            Ok(wallet) => Ok(rocket.manage(wallet)),
            Err(_) => Err(rocket),
        }
    })
}

fn substrate_wallet_firing() -> impl Fairing {
    AdHoc::try_on_ignite("Open substrate wallet", |rocket| async {
        let maybe_wallet = match rocket.state::<AppConfig>() {
            Some(config) => {
                let mnemonic: String =
                    config.mnemonic.parse().expect("Mnemonic is not valid");
                sp_core::sr25519::Pair::from_string(&mnemonic, None)
            }
            None => return Err(rocket),
        };

        match maybe_wallet {
            Ok(wallet) => Ok(rocket.manage(wallet)),
            Err(_) => Err(rocket),
        }
    })
}

fn ethers_providers_firing() -> impl Fairing {
    AdHoc::try_on_ignite("Open ethers provider", |rocket| async {
        let result: Result<HashMap<u64, _>, Error> = match rocket
            .state::<AppConfig>()
        {
            Some(config) => {
                // Supported networks
                let networks = vec![
                    Network::Athena,
                    Network::Hermes,
                    Network::Demeter,
                    Network::TangleEVMTestnet,
                ];

                let mnemonic: String = config.mnemonic.parse().unwrap();
                let wallet = match MnemonicBuilder::<English>::default()
                    .phrase(PathOrString::String(mnemonic))
                    .build()
                {
                    Ok(wallet) => wallet,
                    Err(_) => return Err(rocket),
                };

                let address = wallet.address();
                log::private::info!("Using Account {address:?}");
                let providers: Vec<(_, _)> = networks
                    .iter()
                    .map(|net| net.to_evm_chain_id().unwrap())
                    .map(|chain_id| (chain_id, get_evm_rpc_url(chain_id)))
                    .map(|(chain_id, url)| {
                        let escalator =
                            GeometricGasPrice::new(1.125, 60_u64, None::<u64>);
                        let gas_oracle = GasNow::new();
                        let provider = Provider::<Http>::try_from(url)
                            .unwrap()
                            .wrap_into(|p| {
                                GasEscalatorMiddleware::new(
                                    p,
                                    escalator,
                                    Frequency::PerBlock,
                                )
                            })
                            .gas_oracle(gas_oracle)
                            .with_signer(wallet.clone().with_chain_id(chain_id))
                            .nonce_manager(address);
                        (chain_id, provider)
                    })
                    .collect();

                let mut provider_map: HashMap<u64, _> = HashMap::new();
                for (chain_id, provider) in providers {
                    provider_map.insert(chain_id, Arc::new(provider));
                }
                Ok(provider_map)
            }
            None => return Err(rocket),
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
    AdHoc::try_on_ignite("Open subxt providers", |rocket| async {
        let result: Result<HashMap<u64, OnlineClient<PolkadotConfig>>, Error> =
            match rocket.state::<AppConfig>() {
                Some(_config) => {
                    let networks = vec![Network::Tangle];

                    let mut futures_unordered = FuturesUnordered::new();

                    for network in &networks {
                        let chain_id = network.to_substrate_chain_id().unwrap();
                        let url = get_substrate_rpc_url(chain_id);
                        futures_unordered.push(async move {
                        let res = OnlineClient::<PolkadotConfig>::from_url(url).await;
                        let api = match res {
                            Ok(api) => Some(api),
                            Err(e) => {
                                eprintln!("Error connecting to substrate node: {e}");
                                None
                            }
                        };
                        Result::<_, subxt::Error>::Ok((chain_id, api))
                    });
                    }

                    let mut provider_map: HashMap<
                        u64,
                        OnlineClient<PolkadotConfig>,
                    > = HashMap::new();
                    while let Some(result) = futures_unordered.next().await {
                        match result {
                            Ok((chain_id, Some(api))) => {
                                provider_map.insert(chain_id, api);
                            }
                            Ok((chain_id, None)) => {
                                eprintln!("Skipped connecting to substrate node: {chain_id}");
                            }
                            Err(_e) => return Err(rocket),
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

    // Create the channel
    let (tx_sender, rx_receiver) = mpsc::unbounded_channel();

    // Pass the receiver to your transaction processing system
    TransactionProcessingSystem::new(rx_receiver).run();

    rocket::build()
        .attach(AdHoc::config::<AppConfig>())
        .attach(auth_db_firing())
        .attach(provider_fairing::<auth::providers::Twitter>())
        .attach(ethers_providers_firing())
        .attach(substrate_providers_firing())
        .attach(ethers_wallet_firing())
        .attach(substrate_wallet_firing())
        .attach(cors.to_cors().unwrap())
        .manage(cors.to_cors().unwrap())
        .manage(tx_sender)
        .mount("/", rocket_cors::catch_all_options_routes())
        .mount("/", routes![auth::login::twitter, faucet::faucet])
}
