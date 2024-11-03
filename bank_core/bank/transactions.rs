use std::fmt::Display;

use super::storage::{TransactionAction, TransactionTransfer};

#[derive(Debug, PartialEq, Clone)]
pub struct Transaction {
    pub id: usize,
    pub action: TransactionAction,
    pub account_name: String,
}

impl From<TransactionTransfer> for Transaction {
    fn from(value: TransactionTransfer) -> Self {
        Transaction {
            id: value.id,
            action: value.action,
            account_name: value.account_name,
        }
    }
}

impl Display for Transaction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ID: {}, Action: {:?}", self.id, self.action)
    }
}
