use std::collections::HashMap;

use ethers::types::U256;
use ethers::{
    providers::{Http, Provider},
    types::{Address, TransactionReceipt},
};
use rocket::tokio::sync::oneshot;
use serde::{Deserialize, Serialize};
use webb::substrate::subxt::{utils::AccountId32, OnlineClient, PolkadotConfig};

use crate::error::Error;

pub struct EvmProviders<T> {
    pub providers: HashMap<u64, T>,
}

pub struct SubstrateProviders<T> {
    pub providers: HashMap<u64, T>,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum TxResult {
    Evm(TransactionReceipt),
    Substrate(webb::substrate::subxt::utils::H256),
}

pub enum Transaction {
    Evm {
        provider: Provider<Http>,
        to: Address,
        amount: U256,
        token_address: Option<Address>,
        result_sender: oneshot::Sender<Result<TxResult, Error>>,
    },
    Substrate {
        api: OnlineClient<PolkadotConfig>,
        to: AccountId32,
        amount: u128,
        asset_id: Option<u32>,
        signer: sp_core::sr25519::Pair,
        result_sender: oneshot::Sender<Result<TxResult, Error>>,
    },
}
