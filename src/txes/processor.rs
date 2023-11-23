use std::ops::Div;
use std::sync::Arc;

use crate::subxt::utils::H256;
use ethers::prelude::ContractCall;
use ethers::providers::Middleware;
use ethers::types::{Address, TransactionReceipt, TransactionRequest};
use rocket::tokio::{self, sync::oneshot};
use tokio::sync::mpsc::UnboundedReceiver;
use webb::evm::contract::protocol_solidity::erc20_preset_minter_pauser::ERC20PresetMinterPauserContract;
use webb::evm::ethers;
use webb::evm::ethers::types::U256;
use webb::substrate::subxt::utils::{AccountId32, MultiAddress};
use webb::substrate::subxt::{OnlineClient, PolkadotConfig};
use webb::substrate::tangle_runtime::api as RuntimeApi;

use crate::error::Error;

use super::types::{Transaction, TxResult};

pub struct TransactionProcessingSystem {
    rx_receiver: UnboundedReceiver<Transaction>,
}

impl TransactionProcessingSystem {
    pub fn new(rx_receiver: UnboundedReceiver<Transaction>) -> Self {
        Self { rx_receiver }
    }

    pub fn run(mut self) {
        tokio::spawn(async move {
            println!("Transaction processing system started");
            while let Some(transaction) = self.rx_receiver.recv().await {
                match transaction {
                    Transaction::Evm {
                        provider,
                        to,
                        amount,
                        native_token_amount,
                        token_address,
                        result_sender,
                    } => {
                        let res = handle_evm_tx(
                            provider,
                            to,
                            amount,
                            native_token_amount,
                            token_address,
                            result_sender,
                        )
                        .await;
                        if let Err(e) = res {
                            eprintln!("Error processing EVM transaction: {e}");
                        }
                    }
                    Transaction::Substrate {
                        api,
                        to,
                        native_token_amount,
                        amount,
                        asset_id,
                        signer,
                        result_sender,
                    } => {
                        let res = handle_substrate_tx(
                            api,
                            to,
                            amount,
                            native_token_amount,
                            asset_id,
                            signer,
                            result_sender,
                        )
                        .await;
                        if let Err(e) = res {
                            eprintln!(
                                "Error processing Substrate transaction: {e}"
                            );
                        }
                    }
                }
            }
            eprintln!("Transaction processing system stopped");
        });
    }
}

async fn handle_evm_tx<M: Middleware>(
    provider: M,
    to: Address,
    amount: U256,
    native_token_amount: U256,
    token_address: Option<Address>,
    result_sender: oneshot::Sender<Result<TxResult, Error>>,
) -> Result<TransactionReceipt, Error> {
    if let Some(token_address) = token_address {
        handle_evm_token_tx(provider, to, amount, token_address, result_sender)
            .await
    } else {
        // Only send native token if no token address is provided
        handle_evm_native_tx(provider, to, native_token_amount, result_sender)
            .await
    }
}

async fn handle_evm_native_tx<M: Middleware>(
    provider: M,
    to: Address,
    amount: U256,
    result_sender: oneshot::Sender<Result<TxResult, Error>>,
) -> Result<TransactionReceipt, Error> {
    // Craft the tx
    let has_signer = provider.is_signer().await;
    assert!(has_signer, "Provider must have signer");
    let tx = TransactionRequest::new()
        .to(to)
        .value(amount)
        .gas(U256::from(22000u64)); // TODO: Make this configurable

    let tx_receipt = provider
        .send_transaction(tx, None)
        .await
        .map_err(|e| Error::Custom(e.to_string()))?
        .await
        .map_err(|e| Error::Custom(e.to_string()))?;
    match tx_receipt {
        Some(receipt) => {
            result_sender
                .send(Ok(TxResult::Evm(receipt.clone())))
                .map_err(|e| {
                    Error::Custom(format!("Failed to send receipt: {:?}", e))
                })?;
            Ok(receipt)
        }
        None => {
            result_sender
                .send(Err(Error::Custom(
                    "Failed to send transaction".to_string(),
                )))
                .map_err(|e| {
                    Error::Custom(format!(
                        "Failed to send transaction: {:?}",
                        e
                    ))
                })?;
            Err(Error::Custom("Failed to send transaction".to_string()))
        }
    }
}

async fn handle_evm_token_tx<M: Middleware>(
    provider: M,
    to: Address,
    amount: U256,
    token_address: Address,
    result_sender: oneshot::Sender<Result<TxResult, Error>>,
) -> Result<TransactionReceipt, Error> {
    let has_signer = provider.is_signer().await;
    assert!(has_signer, "Provider must have signer");
    let contract =
        ERC20PresetMinterPauserContract::new(token_address, Arc::new(provider));

    // Fetch the decimals used by the contract so we can compute the decimal amount to send.
    let decimals = contract.decimals().call().await.map_err(|e| {
        Error::Custom(format!("Failed to fetch decimals: {:?}", e))
    })?;
    let decimal_amount = amount * U256::exp10(decimals as usize);

    // Transfer the desired amount of tokens to the `to_address`
    let tx: ContractCall<M, _> = contract.transfer(to, decimal_amount).legacy();
    let pending_tx = tx
        .send()
        .await
        .map_err(|e| Error::Custom(format!("Failed to send tx: {:?}", e)))?;
    match pending_tx
        .await
        .map_err(|e| Error::Custom(format!("Failed to await tx: {:?}", e)))?
    {
        Some(receipt) => {
            result_sender
                .send(Ok(TxResult::Evm(receipt.clone())))
                .map_err(|e| {
                    Error::Custom(format!("Failed to send receipt: {:?}", e))
                })?;
            Ok(receipt)
        }
        None => {
            result_sender
                .send(Err(Error::Custom(
                    "Failed to send transaction".to_string(),
                )))
                .map_err(|e| {
                    Error::Custom(format!(
                        "Failed to send transaction: {:?}",
                        e
                    ))
                })?;
            Err(Error::Custom("Failed to send transaction".to_string()))
        }
    }
}

async fn handle_substrate_tx(
    api: OnlineClient<PolkadotConfig>,
    to: AccountId32,
    _amount: u128,
    native_token_amount: u128,
    asset_id: Option<u32>,
    signer: subxt_signer::sr25519::Keypair,
    result_sender: oneshot::Sender<Result<TxResult, Error>>,
) -> Result<H256, Error> {
    match asset_id {
        Some(asset_id) => Err(Error::Custom(format!(
            "Substrate only supports sending native tokens. Asset ID {} not supported",
            asset_id
        ))),
        None => {
            handle_substrate_native_tx(
                api,
                to,
                native_token_amount,
                signer,
                result_sender,
            )
            .await
        }
    }
}

async fn handle_substrate_native_tx(
    api: OnlineClient<PolkadotConfig>,
    to: AccountId32,
    amount: u128,
    signer: subxt_signer::sr25519::Keypair,
    result_sender: oneshot::Sender<Result<TxResult, Error>>,
) -> Result<H256, Error> {
    let to_address = MultiAddress::Id(to.clone());
    let balance_transfer_tx =
        RuntimeApi::tx().balances().transfer(to_address, amount);
    // Sign and submit the extrinsic.
    let tx_result = api
        .tx()
        .sign_and_submit_then_watch_default(&balance_transfer_tx, &signer)
        .await
        .map_err(|e| Error::Custom(e.to_string()))?;

    let tx_hash = tx_result.extrinsic_hash();

    println!("Tranasction sent with TxHash: {:?}", tx_hash);

    // let events = tx_result
    //     .wait_for_finalized_success()
    //     .await
    //     .map_err(|e| Error::Custom(e.to_string()))?;
    // let block_hash = events.block_hash();

    // // Find a Transfer event and print it.
    // let transfer_event = events
    //     .find_first::<RuntimeApi::balances::events::Transfer>()
    //     .map_err(|e| Error::Custom(e.to_string()))?;
    // if let Some(event) = transfer_event {
    //     let from = event.from;
    //     let to = event.to;
    //     let amount = event.amount.div(10u128.pow(18));
    //     println!("Transfered {amount} tokens {from} -> {to}");
    // }

    // Return the transaction hash.
    result_sender
        .send(Ok(TxResult::Substrate {
            tx_hash,
            block_hash: tx_hash,
        }))
        .map_err(|e| {
            Error::Custom(format!("Failed to send tx_hash: {:?}", e))
        })?;

    Ok(tx_hash)
}
