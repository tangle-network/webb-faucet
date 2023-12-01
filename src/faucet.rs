use chrono::{Duration, Utc};

use rocket::futures::{self, TryFutureExt};
use rocket::http::Status;
use rocket::tokio::sync::mpsc::UnboundedSender;
use rocket::tokio::sync::oneshot;
use rocket::{response::status, serde::json::Json, State};
use serde::Deserialize;
use serde_json::json;

use twitter_v2::{
    authorization::BearerToken, id::NumericId, query::UserField, TwitterApi,
};

use webb::evm::ethers;
use webb::evm::ethers::prelude::k256::ecdsa::SigningKey;
use webb::evm::ethers::signers::Wallet;
use webb::substrate::subxt::OnlineClient;
use webb::substrate::subxt::PolkadotConfig;
use webb_auth::{model::ClaimsData, AuthDb};
use webb_auth_sled::SledAuthDb;

use crate::auth;
use crate::error::Error;
use crate::helpers::address::MultiAddress;
use crate::helpers::files::get_evm_token_address;
use crate::txes::types::{
    EthersClient, EvmProviders, SubstrateProviders, Transaction, TxResult,
};

pub const WEBB_TWITTER_ACCOUNT_ID: u64 = 1355009685859033092;

#[derive(Deserialize, Clone, Debug)]
#[serde(crate = "rocket::serde")]
#[serde(rename_all = "camelCase")]
pub struct Payload {
    faucet: FaucetRequest,
}

// Define the FaucetRequest struct to represent the faucet request data
#[derive(Clone, Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FaucetRequest {
    wallet_address: MultiAddress,
    typed_chain_id: webb_proposals::TypedChainId,
    #[serde(default)]
    only_native_token: bool,
}

pub async fn handle_token_transfer(
    faucet_req: FaucetRequest,
    app_config: &State<crate::AppConfig>,
    evm_providers: &State<EvmProviders<EthersClient>>,
    substrate_providers: &State<
        SubstrateProviders<OnlineClient<PolkadotConfig>>,
    >,
    _evm_wallet: &State<Wallet<SigningKey>>,
    signer_pair: &State<subxt_signer::sr25519::Keypair>,
    tx_sender: &State<UnboundedSender<Transaction>>,
) -> Result<TxResult, Error> {
    let (result_sender, result_receiver) = oneshot::channel();
    match faucet_req.typed_chain_id {
        webb_proposals::TypedChainId::Evm(chain_id) => {
            // Create a provider for the chain id and instantiate the contract.
            let provider = evm_providers
                .providers
                .get(&chain_id.into())
                .ok_or(Error::Custom(format!(
                    "No provider found for chain id {}",
                    chain_id
                )))?
                .clone();
            let token_address = if faucet_req.only_native_token {
                None
            } else {
                Some(get_evm_token_address(chain_id.into()).into())
            };
            let dest = *faucet_req.wallet_address.ethereum().unwrap();

            // Send transaction to the processor.
            tx_sender
                .send(Transaction::Evm {
                    provider,
                    to: dest,
                    amount: app_config.token_amount.into(),
                    native_token_amount: ethers::utils::parse_ether(
                        app_config.native_token_amount,
                    )
                    .expect("Failed to parse native token amount"),
                    token_address,
                    result_sender,
                })
                .expect("Failed to send transaction to processor");
        }
        webb_proposals::TypedChainId::Substrate(chain_id) => {
            // 1. Create a provider for the chain id.
            let api = substrate_providers
                .providers
                .get(&chain_id.into())
                .ok_or(Error::Custom(format!(
                    "No provider found for chain id {}",
                    chain_id
                )))?
                .clone();

            // 2. Build a balance transfer extrinsic.
            let dest = faucet_req.wallet_address.substrate().unwrap().clone();
            tx_sender
                .send(Transaction::Substrate {
                    api,
                    to: dest,
                    amount: app_config.token_amount.into(),
                    native_token_amount: ethers::utils::parse_ether(
                        app_config.native_token_amount,
                    )
                    .expect("Failed to parse native token amount")
                    .as_u128(),
                    asset_id: None,
                    signer: signer_pair.inner().clone(),
                    timeout: std::time::Duration::from_millis(
                        app_config.tx_timeout,
                    ),
                    result_sender,
                })
                .expect("Failed to send transaction to processor");
        }
        _ => return Err(Error::Custom("Invalid chain id".to_string())),
    };

    // await the result
    let result = match result_receiver.await {
        Ok(res) => match res {
            Ok(tx_result) => tx_result, // if transaction execution was successful
            Err(e) => return Err(e), // if transaction execution resulted in an error
        },
        Err(e) => {
            return Err(Error::Custom(format!(
                "Transaction was not processed: {}",
                e
            )))
        }
    };

    // proceed with your result
    Ok(result)
}

pub async fn check_twitter(
    app_config: &State<crate::AppConfig>,
    twitter_bearer_token: auth::TwitterBearerToken<'_>,
) -> Result<twitter_v2::User, Error> {
    // during debug builds, we return a dummy user
    if cfg!(debug_assertions) {
        let token = twitter_bearer_token.token();
        return Ok(twitter_v2::User {
            id: NumericId::new(token.parse().unwrap()),
            username: "dummy".to_string(),
            name: "dummy".to_string(),
            created_at: None,
            description: None,
            entities: None,
            location: None,
            pinned_tweet_id: None,
            profile_image_url: None,
            protected: None,
            public_metrics: None,
            url: None,
            verified: None,
            withheld: None,
        });
    }
    let auth = BearerToken::new(twitter_bearer_token.token());
    let twitter_api = TwitterApi::new(auth);

    let twitter_user: twitter_v2::User = twitter_api
        .get_users_me()
        .send()
        .inspect_err(|e| println!("Error getting user: {:?}", e))
        .and_then(|res| {
            let res = res
                .data()
                .cloned()
                .ok_or_else(|| {
                    twitter_v2::error::Error::Custom(
                        "No user found".to_string(),
                    )
                })
                .map_err(Into::into);
            futures::future::ready(res)
        })
        .await?;

    println!("Twitter User: {:#?}", twitter_user.username);

    let is_following_webb = if app_config.verify_following_webb {
        let mut is_following_webb = false;
        let mut maybe_pagination_token: Option<String> = None;
        let mut is_first_page = true;

        // Check if the user is following the webb twitter account
        while is_first_page
            || !is_following_webb && maybe_pagination_token.is_some()
        {
            // Check if the user is following the webb twitter account
            // - the account username is `webbprotocol`
            // - the user id is `1355009685859033092`
            let mut get_my_following_req =
                twitter_api.with_user_ctx().await?.get_my_following();

            let mut req = get_my_following_req
                .user_fields([UserField::Id])
                .max_results(100);
            if let Some(ref token) = maybe_pagination_token {
                req = req.pagination_token(token);
            }

            let my_followers = req.send().await;
            // Check if the user is following the webb twitter account and return
            // an error if they are not. If successful, return a bool and a pagination token.
            // The pagination token is used to get the next page of followers.
            let (is_following_webb_, maybe_pagination_token_) =
                match my_followers {
                    Ok(followers) => {
                        // Get number of followers
                        let num_followers = followers
                            .data
                            .as_ref()
                            .map(Vec::len)
                            .unwrap_or_default();
                        let next_token =
                            followers.meta.clone().and_then(|m| m.next_token);
                        println!(
                            "Got {} followers, next token: {:?}",
                            num_followers, next_token
                        );

                        let webb_user_id =
                            NumericId::new(WEBB_TWITTER_ACCOUNT_ID);
                        (
                            followers
                                .data
                                .clone()
                                .map(|u| {
                                    u.iter().any(|follower| {
                                        follower.id == webb_user_id
                                    })
                                })
                                .unwrap_or(false),
                            next_token,
                        )
                    }
                    Err(e) => return Err(Error::TwitterError(e)),
                };

            is_following_webb = is_following_webb_;
            maybe_pagination_token = maybe_pagination_token_;
            is_first_page = false;
        }
        is_following_webb
    } else {
        // Skip the verification step
        println!("Skipping verification step");
        true
    };

    println!(
        "{:?} User {:?} is following webb: {:?}",
        Utc::now().to_rfc3339(),
        twitter_user.username,
        is_following_webb
    );

    if !is_following_webb {
        Err(Error::Custom(
            "User is not following the webb twitter account".to_string(),
        ))
    } else {
        Ok(twitter_user)
    }
}

#[post("/faucet", data = "<payload>")]
#[allow(clippy::too_many_arguments)]
pub async fn faucet(
    app_config: &State<crate::AppConfig>,
    twitter_bearer_token: auth::TwitterBearerToken<'_>,
    payload: Json<Payload>,
    auth_db: &State<SledAuthDb>,
    evm_providers: &State<EvmProviders<EthersClient>>,
    substrate_providers: &State<
        SubstrateProviders<OnlineClient<PolkadotConfig>>,
    >,
    evm_wallet: &State<Wallet<SigningKey>>,
    signer_pair: &State<subxt_signer::sr25519::Keypair>,
    tx_sender: &State<UnboundedSender<Transaction>>,
) -> Result<status::Custom<String>, Error> {
    let twitter_user = check_twitter(app_config, twitter_bearer_token).await?;
    let faucet_data = payload.clone().into_inner().faucet;
    // Extract faucet request fields
    let FaucetRequest {
        wallet_address,
        typed_chain_id,
        ..
    } = faucet_data.clone();
    println!(
        "Requesting faucet for (address {}, chain: {:?}",
        wallet_address, typed_chain_id
    );
    // Check if the user's last claim date is within the last 24 hours
    let claim_data = auth_db
        .get_last_claim_data(twitter_user.id.into(), typed_chain_id)
        .await?;
    let last_claim_date = claim_data.map(|c| c.last_claimed_date);
    let now = Utc::now();
    if let Some(last_claim_date) = last_claim_date {
        let time_delay =
            Duration::from_std(app_config.time_to_wait_between_claims)
                .expect("valid duration");
        if now < last_claim_date + time_delay {
            println!(
                "{:?} User {:?} tried to claim again before the time limit",
                Utc::now().to_rfc3339(),
                twitter_user.username
            );
            return Ok(status::Custom(
                Status::UnprocessableEntity,
                json!({
                    "error": "Error claiming tokens",
                    "reason": "You can't claim right now. Please try again later.",
                    "wallet": wallet_address,
                    "typed_chain_id": typed_chain_id,
                    "last_claimed_date": last_claim_date,
                    "time_to_wait_between_claims_ms": app_config.time_to_wait_between_claims.as_millis(),
                    "user": twitter_user,
                })
                .to_string(),
            ));
        }
    }

    println!(
        "Paying {} ({wallet_address}) on chain: {typed_chain_id:?}",
        twitter_user.username,
    );

    match handle_token_transfer(
        faucet_data,
        app_config,
        evm_providers,
        substrate_providers,
        evm_wallet,
        signer_pair,
        tx_sender,
    )
    .await
    {
        Ok(tx_result) => {
            let claim: ClaimsData = ClaimsData {
                identity: twitter_user.id.into(),
                address: wallet_address.clone().into(),
                last_claimed_date: now,
            };

            auth_db
                .put_last_claim_data(
                    twitter_user.id.into(),
                    typed_chain_id,
                    claim,
                )
                .await?;
            println!(
                "{:?} Paid {} on chain: {:?}",
                Utc::now().to_rfc3339(),
                wallet_address,
                typed_chain_id
            );
            Ok(status::Custom(
                Status::Ok,
                json!({
                    "wallet": wallet_address,
                    "typed_chain_id": typed_chain_id,
                    "last_claimed_date": now,
                    "user": twitter_user,
                    "tx_result": tx_result,
                })
                .to_string(),
            ))
        }
        Err(e) => {
            rocket::log::private::error!("Error transferring tokens: {e:?}");
            Ok(status::Custom(
                Status::InternalServerError,
                json!({
                    "error": "Error transferring tokens",
                    "reason": format!("{e}"),
                    "typed_chain_id": typed_chain_id,
                    "wallet": wallet_address,
                    "user": twitter_user,
                    "last_claimed_date": now,
                })
                .to_string(),
            ))
        }
    }
}
