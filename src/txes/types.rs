use super::networks::Network;
use std::collections::HashMap;
use webb::evm::ethers::{prelude::k256::U256, types::Address};

pub struct EvmProviders<T> {
    pub providers: HashMap<u64, T>,
}

pub struct SubstrateProviders<T> {
    pub providers: HashMap<u64, T>,
}

pub enum Transaction {
    EVM(EvmTransaction),
    Substrate(SubstrateTransaction),
}

pub struct EvmTransaction {
    // Here you can add the necessary details for an EVM transaction
    // For example, the following are commonly included in a transaction
    pub network: Network,
    pub from: Address,
    pub to: Address,
    pub value: U256,
    pub gas_price: U256,
    pub gas: U256,
    pub nonce: U256,
    pub data: Vec<u8>,
}

pub struct SubstrateTransaction {
    // Here you can add the necessary details for a Substrate transaction
    // Details will be specific to your use case
}
