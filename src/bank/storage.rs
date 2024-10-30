use std::fmt::Display;
use thiserror::Error as TError;

// data between database and Model
#[derive(Debug, PartialEq, Eq)]
pub struct AccountTransfer {
    pub name: String,
    pub balance: usize,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TransactionAction {
    Registration,
    Increment(usize),
    Decrement(usize),
}

#[derive(Debug)]
pub struct TransactionTransfer {
    pub id: usize,
    pub action: TransactionAction,
    pub account_name: String,
}

impl AccountTransfer {
    pub fn new(name: String, balance: Option<usize>) -> Self {
        Self {
            name,
            balance: balance.unwrap_or_default(),
        }
    }
}

impl Clone for AccountTransfer {
    fn clone(&self) -> Self {
        Self {
            name: self.name.clone(),
            balance: self.balance,
        }
    }
}

#[derive(TError, Debug, PartialEq, Eq)]
pub enum Error {
    #[error("storage error: `{0}`")]
    StorageError(String),
    #[error("account already exists")]
    AccountAlreadyExists,
    #[error("account not exists")]
    AccountNotExists,
    #[error("transaction not exists")]
    TransactionNotExists,
}

pub trait AccountStorage {
    // creates a new account if not exists
    // Errors: AccountAlreadyExists, StorageError
    fn create_account(&mut self, raw_data: AccountTransfer) -> Result<AccountTransfer, Error>;

    // gets account from storage if exists
    fn get_account(&self, name: String) -> Result<AccountTransfer, Error>;

    // updates account data in storage
    fn update_account(&mut self, transfer_data: AccountTransfer) -> Result<AccountTransfer, Error>;

    // returns special fee account to store money from transactions
    fn fee_account(&self) -> Result<AccountTransfer, Error>;

    // returns list of accounts
    fn accounts(&self) -> Result<Vec<AccountTransfer>, Error>;
}

pub trait TransactionStorage {
    fn create_transaction(
        &mut self,
        account_name: String,
        action: TransactionAction,
    ) -> Result<TransactionTransfer, Error>;
    fn transactions(&self) -> Result<Vec<TransactionTransfer>, Error>;
    fn account_transactions(&self, account_name: String)
        -> Result<Vec<TransactionTransfer>, Error>;
    fn transaction_by_id(&self, id: usize) -> Result<TransactionTransfer, Error>;
}

impl Display for TransactionTransfer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.action {
            TransactionAction::Registration => {
                write!(f, "ID: {}, Action: {:?}", self.id, self.action)
            }
            TransactionAction::Increment(amount) => {
                write!(
                    f,
                    "ID: {}, Action: {:?}, Amount: {}",
                    self.id, self.action, amount
                )
            }
            TransactionAction::Decrement(amount) => {
                write!(
                    f,
                    "ID: {}, Action: {:?}, Amount: {}",
                    self.id, self.action, amount
                )
            }
        }
    }
}
