use chrono::{Days, Utc};
use rocket::futures::{self, TryFutureExt};
use rocket::{response::status, serde::json::Json, State};
use serde::Deserialize;
use serde::Serialize;
use serde_json::json;
use twitter_v2::{authorization::BearerToken, id::NumericId, query::UserField, TwitterApi};
use webb_auth::model::UniversalWalletAddress;
use webb_auth::{model::ClaimsData, AuthDb};
use webb_auth_sled::SledAuthDb;

use crate::auth;
use crate::error::Error;

const WEBB_TWITTER_ACCOUNT_ID: u64 = 1355009685859033092;

#[derive(Deserialize, Clone, Debug)]
#[serde(crate = "rocket::serde")]
#[serde(rename_all = "camelCase")]
pub struct Payload {
    faucet: FaucetRequest,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(crate = "rocket::serde")]
#[serde(tag = "type", content = "value", rename_all = "camelCase")]
pub enum MultiAddress {
    Ethereum(webb::evm::ethers::types::Address),
    Substrate(webb::substrate::subxt::utils::AccountId32),
}

impl MultiAddress {
    /// Returns `true` if the multi address is [`Ethereum`].
    ///
    /// [`Ethereum`]: MultiAddress::Ethereum
    #[must_use]
    pub fn is_ethereum(&self) -> bool {
        matches!(self, Self::Ethereum(..))
    }

    /// Returns `true` if the multi address is [`Substrate`].
    ///
    /// [`Substrate`]: MultiAddress::Substrate
    #[must_use]
    pub fn is_substrate(&self) -> bool {
        matches!(self, Self::Substrate(..))
    }
}

impl core::fmt::Display for MultiAddress {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Ethereum(address) => write!(f, "{}", address),
            Self::Substrate(address) => write!(f, "{}", address),
        }
    }
}

impl From<MultiAddress> for UniversalWalletAddress {
    fn from(multi_address: MultiAddress) -> Self {
        match multi_address {
            MultiAddress::Ethereum(address) => Self::Ethereum(address.to_fixed_bytes()),
            MultiAddress::Substrate(address) => Self::Substrate(address.0),
        }
    }
}

// Define the FaucetRequest struct to represent the faucet request data
#[derive(Clone, Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FaucetRequest {
    wallet_address: MultiAddress,
    typed_chain_id: webb_proposals::TypedChainId,
}

#[post("/faucet", data = "<payload>")]
pub async fn faucet(
    twitter_bearer_token: auth::TwitterBearerToken<'_>,
    payload: Json<Payload>,
    auth_db: &State<SledAuthDb>,
) -> Result<status::Accepted<String>, Error> {
    let faucet_data = payload.clone().into_inner().faucet;
    let auth = BearerToken::new(twitter_bearer_token.token());
    let twitter_api = TwitterApi::new(auth);
    // Extract faucet request fields
    let FaucetRequest {
        wallet_address,
        typed_chain_id,
    } = faucet_data;
    println!(
        "Requesting faucet for (address {}, chain: {:?}",
        wallet_address, typed_chain_id
    );
    let twitter_user: twitter_v2::User = twitter_api
        .get_users_me()
        .send()
        .inspect_err(|e| println!("Error getting user: {:?}", e))
        .and_then(|res| {
            let res = res
                .data()
                .cloned()
                .ok_or_else(|| twitter_v2::error::Error::Custom("No user found".to_string()))
                .map_err(Into::into);
            futures::future::ready(res)
        })
        .await?;

    println!("Twitter User: {:#?}", twitter_user.username);

    let mut is_following_webb = false;
    let mut maybe_pagination_token: Option<String> = None;
    let mut is_first_page = true;

    // Check if the user is following the webb twitter account
    while is_first_page || !is_following_webb && maybe_pagination_token.is_some() {
        // Check if the user is following the webb twitter account
        // - the account username is `webbprotocol`
        // - the user id is `1355009685859033092`
        let mut get_my_following_req = twitter_api.with_user_ctx().await?.get_my_following();

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
        let (is_following_webb_, maybe_pagination_token_) = match my_followers {
            Ok(followers) => {
                // Get number of followers
                let num_followers = followers.data.as_ref().map(|u| u.len()).unwrap_or_default();
                let next_token = followers.meta.clone().and_then(|m| m.next_token);
                println!(
                    "Got {} followers, next token: {:?}",
                    num_followers.to_string(),
                    next_token
                );

                let webb_user_id = NumericId::new(WEBB_TWITTER_ACCOUNT_ID);
                (
                    followers
                        .data
                        .clone()
                        .map(|u| u.iter().any(|follower| follower.id == webb_user_id))
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

    println!(
        "{:?} User {:?} is following webb: {:?}",
        Utc::now().to_rfc3339(),
        twitter_user.username,
        is_following_webb
    );

    if !is_following_webb {
        return Err(Error::Custom(
            "User is not following the webb twitter account".to_string(),
        ));
    }

    // Check if the user's last claim date is within the last 24 hours
    let claim_data = auth_db
        .get_last_claim_data(twitter_user.id.into(), typed_chain_id)
        .await?;
    let last_claim_date = claim_data.map(|c| c.last_claimed_date);
    let now = Utc::now();
    // check if the rust env is test, if so, skip the 24 hour check
    let rust_env = std::env::var("ROCKET_PROFILE").unwrap_or_default();
    if rust_env == "release" {
        if let Some(last_claim_date) = last_claim_date {
            if last_claim_date <= now.checked_add_days(Days::new(1)).unwrap() {
                println!(
                    "{:?}  User {:?} tried to claim again before 24 hours",
                    Utc::now().to_rfc3339(),
                    twitter_user.username
                );
                return Err(Error::Custom(
                    "You can only claim once every 24 hours.".to_string(),
                ));
            }
        }
    }

    let claim: ClaimsData = ClaimsData {
        identity: twitter_user.id.into(),
        address: wallet_address.clone().into(),
        last_claimed_date: now,
    };

    auth_db
        .put_last_claim_data(twitter_user.id.into(), typed_chain_id, claim)
        .await?;
    // Process the claim and build the response
    println!(
        "{:?}  Claiming for user: {:?}",
        Utc::now().to_rfc3339(),
        twitter_user.username,
    );
    println!(
        "{:?} Paying {} on chain: {:?}",
        Utc::now().to_rfc3339(),
        wallet_address,
        typed_chain_id
    );
    // TODO: Handle tx and return the hash
    let tx_hash = "0x1234";
    Ok(status::Accepted(Some(
        json!({
            "wallet": wallet_address,
            "typed_chain_id": typed_chain_id,
            "last_claimed_date": now,
            "user": twitter_user,
            "tx_hash": tx_hash
        })
        .to_string(),
    )))
}
