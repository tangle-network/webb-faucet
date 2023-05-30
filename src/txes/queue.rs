use std::collections::VecDeque;

use super::types::Transaction;

pub struct TransactionQueue {
    queue: VecDeque<Transaction>,
}

impl TransactionQueue {
    pub fn new() -> Self {
        Self {
            queue: VecDeque::new(),
        }
    }

    pub fn push(&mut self, transaction: Transaction) {
        self.queue.push_back(transaction);
    }

    pub fn pop(&mut self) -> Option<Transaction> {
        self.queue.pop_front()
    }
}
