use std::{cell::RefCell, fmt::Display};

// data between database and Model
#[derive(Debug, PartialEq, Eq)]
pub struct AccountTransfer {
    pub name: String,
    pub balance: usize,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TransactionAction {
    Registration,
    Increment,
    Decrement,
}

#[derive(Debug)]
pub struct TransactionTransfer {
    pub id: usize,
    pub action: TransactionAction,
    pub amount: usize,
    pub account_name: String,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct StorageConnectionError {
    value: String,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum StorageCreateAccountError {
    StorageError(String),
    AccountAlreadyExists,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum StorageGetAccountError {
    StorageError(String),
    AccountNotExists,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum StorageUpdateAccountError {
    StorageError(String),
    AccountNotExists,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum StorageCreateTransactionError {
    StorageError(String),
    AccountNotExists,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum GetTransactionsError {
    StorageError(String),
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum GetTransactionError {
    StorageError(String),
    AccountNotExists,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum GetTransactionByIdError {
    StorageError(String),
    NotFound,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum FeeAccountError {
    StorageError(String),
}

#[derive(Debug, PartialEq, Eq)]
pub struct Storage<A: AccountStorage, T: TransactionStorage> {
    pub acc_storage: RefCell<A>,
    pub tr_storage: RefCell<T>,
}

impl Clone for AccountTransfer {
    fn clone(&self) -> Self {
        Self {
            name: self.name.clone(),
            balance: self.balance,
        }
    }
}

pub trait AccountStorage {
    // creates a new account if not exists (if exists returns None)
    fn create_account(
        &mut self,
        raw_data: AccountTransfer,
    ) -> Result<AccountTransfer, StorageCreateAccountError>;

    // gets account from storage if exists
    fn get_account(&self, name: String) -> Result<AccountTransfer, StorageGetAccountError>;

    // updates account data in storage
    fn update_account(
        &mut self,
        transfer_data: AccountTransfer,
    ) -> Result<AccountTransfer, StorageUpdateAccountError>;

    // returns special fee account to store money from transactions
    fn fee_account(&mut self) -> Result<AccountTransfer, FeeAccountError>;
}

pub trait TransactionStorage {
    fn create_transaction(
        &mut self,
        account_name: String,
        amount: usize,
        action: TransactionAction,
    ) -> Result<TransactionTransfer, StorageCreateTransactionError>;
    fn transactions(&self) -> Result<Vec<TransactionTransfer>, GetTransactionsError>;
    fn account_transactions(
        &self,
        account_name: String,
    ) -> Result<Vec<TransactionTransfer>, GetTransactionError>;
    fn get_transaction_by_id(
        &self,
        id: usize,
    ) -> Result<TransactionTransfer, GetTransactionByIdError>;
}

impl<A: AccountStorage, T: TransactionStorage> Storage<A, T> {
    pub fn new(acc_storage: A, tr_storage: T) -> Self {
        Storage {
            acc_storage: RefCell::new(acc_storage),
            tr_storage: RefCell::new(tr_storage),
        }
    }
}

impl Display for TransactionTransfer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "ID: {}, Action: {:?}, Amount: {}",
            self.id, self.action, self.amount
        )
    }
}
