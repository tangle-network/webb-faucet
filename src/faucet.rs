use chrono::{Days, Utc};
use rocket::{response::status, serde::json::Json, State};
use serde::Deserialize;
use serde_json::json;
use twitter_v2::{authorization::BearerToken, id::NumericId, query::UserField, TwitterApi};
use webb_auth::{model::ClaimsData, AuthDb};
use webb_auth_sled::SledAuthDb;
use webb_proposals::TypedChainId;

use crate::error::Error;

#[derive(Deserialize, Clone, Debug)]
#[serde(crate = "rocket::serde")]
pub struct Payload {
    faucet: FaucetRequest,
    oauth: OAuth2Token,
}

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
pub struct OAuth2Token {
    access_token: String,
}

// Define the FaucetRequest struct to represent the faucet request data
#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
pub struct FaucetRequest {
    address: String,
    typed_chain_id: String,
}

#[post("/faucet", data = "<payload>")]
pub async fn faucet(
    payload: Json<Payload>,
    mut connection: &State<sled::Db>,
) -> Result<status::Accepted<String>, Error> {
    println!("Faucet request: {:?}", payload.clone().into_inner());
    let faucet_data = payload.clone().into_inner().faucet.clone();
    let oauth2_data = payload.into_inner().oauth.clone();
    let auth = BearerToken::new(oauth2_data.access_token);
    let twitter_api = TwitterApi::new(auth);
    // Extract faucet request fields
    let FaucetRequest {
        address,
        typed_chain_id,
    } = faucet_data;

    let typed_chain_id = TypedChainId::from(typed_chain_id.parse::<u64>().unwrap_or_default());
    println!(
        "Requesting faucet for (address {:?}, chain: {:?}",
        address, typed_chain_id
    );
    let maybe_user: Option<twitter_v2::data::User> = twitter_api
        .get_users_me()
        .send()
        .await
        .map_err(|e| {
            println!("Maybe user error: {:?}", e);
            Error::TwitterError(e)
        })
        .map(|res| match res.data.clone() {
            Some(data) => Some(data),
            None => None,
        })?;

    println!("Maybe user: {:#?}", maybe_user);
    // Throw an error if the user is not found
    let user = if maybe_user.is_none() {
        return Err(Error::TwitterError(twitter_v2::error::Error::Custom(
            "No user found".to_string(),
        )));
    } else {
        maybe_user.unwrap()
    };

    // Check if the user is following the webb twitter account
    // - the account username is `webbprotocol`
    // - the user id is `1355009685859033092`
    let my_followers = twitter_api
        .with_user_ctx()
        .await?
        .get_my_following()
        .user_fields([UserField::Id])
        .max_results(100)
        .send()
        .await;

    // Check if the user is following the webb twitter account and return
    // an error if they are not. If successful, return a bool and a pagination token.
    // The pagination token is used to get the next page of followers.
    let (mut is_following_webb, mut maybe_pagination_token) = match my_followers {
        Ok(followers) => {
            let webb_user_id = NumericId::new(1355009685859033092);
            (
                followers
                    .data
                    .clone()
                    .map(|u| u.iter().any(|follower| follower.id == webb_user_id))
                    .unwrap_or(false),
                followers.meta.clone().map(|m| m.next_token),
            )
        }
        Err(e) => (false, None),
    };

    // If the user is not following the webb twitter account, check if there is a
    // pagination token. If there is, get the next page of followers and check if
    // the user is following the webb twitter account. If there is no pagination
    // token, it means there are no more pages. Loop until we exhaust all pages.
    if !is_following_webb {
        while maybe_pagination_token.is_some()
            && maybe_pagination_token.clone().unwrap().is_some()
        {
            let my_followers = twitter_api
                .with_user_ctx()
                .await?
                .get_my_following()
                .user_fields([UserField::Id])
                .max_results(100)
                .pagination_token(maybe_pagination_token.unwrap().unwrap().as_ref())
                .send()
                .await;

            let (maybe_following, new_pagination_token) = match my_followers {
                Ok(followers) => {
                    println!(
                        "Follower count: {:?}",
                        followers.data.clone().map(|u| u.len())
                    );
                    println!(
                        "Pagination token: {:?}",
                        followers.meta.clone().map(|m| m.next_token)
                    );
                    let webb_user_id = NumericId::new(1355009685859033092);
                    (
                        followers
                            .data
                            .clone()
                            .map(|u| u.iter().any(|follower| follower.id == webb_user_id))
                            .unwrap_or(false),
                        followers.meta.clone().map(|m| m.next_token),
                    )
                }
                Err(_) => (false, None),
            };

            maybe_pagination_token = new_pagination_token;
            is_following_webb = is_following_webb || maybe_following;
        }

        if !is_following_webb {
            return Err(Error::Custom("User is not following webb".to_string()));
        }
    }

    println!(
        "{:?}  User {:?} is following webb: {:?}",
        Utc::now().to_rfc3339(),
        user,
        is_following_webb
    );

    // Check if the user's last claim date is within the last 24 hours
    let last_claim_date = <SledAuthDb as AuthDb>::get_last_claim_date(
        &mut connection,
        user.id.into(),
        typed_chain_id.clone(),
    )
    .await
    .map_err(|e| {
        println!("Last claim date error: {:?}", e);
        Error::Custom(format!("Error: {:?}", e.to_string()))
    })?;

    let now = Utc::now();
    // check if the rust env is test, if so, skip the 24 hour check
    let rust_env = std::env::var("RUST_ENV").unwrap_or_default();
    if rust_env == "production" {
        if let Some(last_claim_date) = last_claim_date {
            if last_claim_date <= now.checked_add_days(Days::new(1)).unwrap() {
                println!(
                    "{:?}  User {:?} tried to claim again before 24 hours",
                    Utc::now().to_rfc3339(),
                    user
                );
                return Err(Error::Custom(format!(
                    "You can only claim once every 24 hours.",
                )));
            }
        }
    }

    let claim: ClaimsData = ClaimsData {
        identity: user.id.into(),
        address: address.clone(),
        last_claimed_date: now,
    };

    <SledAuthDb as AuthDb>::put_last_claim_date(connection, user.id.into(), typed_chain_id, claim)
        .await
        .map_err(|e| {
            println!("Put last claim date error: {:?}", e);
            Error::Custom(format!("Error: {:?}", e.to_string()))
        })?;

    // Process the claim and build the response
    println!(
        "{:?}  Claiming for user: {:?}",
        Utc::now().to_rfc3339(),
        user
    );
    println!(
        "{:?}  Paying {:?} on chain: {:?}",
        Utc::now().to_rfc3339(),
        address,
        typed_chain_id
    );
    // TODO: Handle tx and return the hash
    let tx_hash = "0x1234";
    return Ok(status::Accepted(Some(
        json!({
            "address": address,
            "typed_chain_id": typed_chain_id,
            "last_claimed_date": now,
            "user": user,
            "tx_hash": tx_hash
        })
        .to_string(),
    )));
}
