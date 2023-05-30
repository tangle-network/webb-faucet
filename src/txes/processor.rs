use rocket::tokio::task;

use super::{
    queue::TransactionQueue,
    types::{EvmTransaction, SubstrateTransaction, Transaction},
};

pub async fn process_transactions(transaction_queue: &mut TransactionQueue) {
    loop {
        if let Some(transaction) = transaction_queue.pop() {
            let handle = match transaction {
                Transaction::EVM(evmt) => task::spawn(process_evm_transaction(evmt)),
                Transaction::Substrate(subt) => task::spawn(process_substrate_transaction(subt)),
            };

            let _ = handle.await;
        } else {
            break;
        }
    }
}

async fn process_evm_transaction(evmt: EvmTransaction) {
    // implement the logic to process an EVM transaction here
    // probably you'll use the ethers library for this
}

async fn process_substrate_transaction(subt: SubstrateTransaction) {
    // implement the logic to process a Substrate transaction here
    // probably you'll use the subxt library for this
}
