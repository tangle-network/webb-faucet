use chrono::{Days, Utc};
use rocket::{response::status, serde::json::Json, State};
use serde::Deserialize;
use serde_json::json;
use twitter_v2::{authorization::BearerToken, TwitterApi};
use webb_auth::{model::ClaimsData, AuthDb};
use webb_auth_sled::SledAuthDb;
use webb_proposals::TypedChainId;

use crate::error::Error;

#[derive(Deserialize, Clone, Debug)]
#[serde(crate = "rocket::serde")]
pub struct Payload {
    faucet: FaucetRequest,
    oauth2: OAuth2Token,
}

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
pub struct OAuth2Token {
    access_token: String,
}

// Define the FaucetRequest struct to represent the faucet request data
#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
pub struct FaucetRequest {
    address: String,
    typed_chain_id: TypedChainId,
    twitter_id: u64,
}

#[post("/faucet", data = "<payload>")]
pub async fn faucet(
    payload: Json<Payload>,
    mut connection: &State<sled::Db>,
) -> Result<status::Accepted<String>, Error> {
    let faucet_data = payload.clone().into_inner().faucet.clone();
    let oauth2_data = payload.into_inner().oauth2.clone();
    let auth = BearerToken::new(oauth2_data.access_token);
    let twitter_api = TwitterApi::new(auth);
    // Extract faucet request fields
    let FaucetRequest {
        address,
        typed_chain_id,
        twitter_id,
    } = faucet_data;

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

    // Throw an error if the user is not found
    if maybe_user.is_none() {
        return Err(Error::TwitterError(twitter_v2::error::Error::Custom(
            "No user found".to_string(),
        )));
    }

    let user = maybe_user.unwrap();

    // Check if the user's twitter id matches the one in the request
    if user.id != twitter_id {
        return Err(Error::TwitterError(twitter_v2::error::Error::Custom(
            "Twitter id does not match".to_string(),
        )));
    }

    // Check if the user's last claim date is within the last 24 hours
    let last_claim_date = <SledAuthDb as AuthDb>::get_last_claim_date(
        &mut connection,
        user.id.into(),
        typed_chain_id.clone(),
    )
    .await
    .map_err(|e| {
        println!("error: {:?}", e);
        Error::Custom(format!("Error: {:?}", e.to_string()))
    })?;

    let now = Utc::now();
    if let Some(last_claim_date) = last_claim_date {
        if last_claim_date <= now.checked_add_days(Days::new(1)).unwrap() {
            return Err(Error::Custom(format!(
                "You can only claim once every 24 hours.",
            )));
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
            println!("error: {:?}", e);
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
    return Ok(status::Accepted(Some(
        json!({
            "address": address,
            "typed_chain_id": typed_chain_id,
            "last_claimed_date": now,
            "user": user,
        })
        .to_string(),
    )));
}
