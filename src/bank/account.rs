use std::fmt::Display;

use crate::bank::storage::{
    AccountStorage, AccountTransfer, TransactionAction, TransactionStorage,
};
use thiserror::Error as TError;

use super::{
    storage::Error as StorageError,
    transactions::Transaction,
};

#[derive(Debug, Default)]
pub struct Account {
    pub balance: usize,
    pub name: String,
    pub trs: Vec<usize>,
}

impl Display for Account {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Account: {}. Balance: {}", self.name, self.balance)
    }
}

#[derive(TError, Debug, PartialEq, Eq)]
pub enum Error {
    #[error("storage error: `{0}`")]
    Storage(String),
    #[error("account already exists")]
    AccountAlreadyExists,
    #[error("account not exists")]
    AccountNotExists,
    #[error("empty transaction")]
    EmptyTransaction,
    #[error("not enough money")]
    NotEnoughMoney,
    #[error("transaction not exists")]
    TransactionNotExists,
}

impl From<StorageError> for Error {
    fn from(value: StorageError) -> Self {
        match value {
            StorageError::StorageError(v) => Error::Storage(v),
            StorageError::AccountAlreadyExists => Error::AccountAlreadyExists,
            StorageError::AccountNotExists => Error::AccountNotExists,
            StorageError::TransactionNotExists => Error::TransactionNotExists,
        }
    }
}

impl From<AccountTransfer> for Account {
    fn from(value: AccountTransfer) -> Self {
        Account {
            name: value.name,
            balance: value.balance,
            trs: value.trs,
        }
    }
}

impl From<&Account> for AccountTransfer {
    fn from(value: &Account) -> Self {
        AccountTransfer {
            name: value.name.clone(),
            balance: value.balance,
            trs: value.trs.clone(),
        }
    }
}

impl Account {
    // task 1 create an account
    // create an account
    // errors: AccountAlreadyExists, Storage
    pub fn new<S: AccountStorage, T: TransactionStorage>(
        name: String,
        acc_storage: &mut S,
        tr_storage: &mut T,
    ) -> Result<Account, Error> {
        acc_storage.create_account(AccountTransfer::new(name.clone(), None))?;
        tr_storage.create_transaction(name.clone(), TransactionAction::Registration)?;
        Ok(Account {
            name,
            balance: Default::default(),
            trs: Vec::new(),
        })
    }

    // task 2 part 1
    // increments an account balance
    // errors: EmptyTransaction, Storage, AccountNotExists
    pub fn inc_balance<S: AccountStorage, T: TransactionStorage>(
        &mut self,
        value: usize,
        acc_storage: &mut S,
        tr_storage: &mut T,
    ) -> Result<Transaction, Error> {
        if value == 0 {
            return Err(Error::EmptyTransaction);
        }

        let mut acc_tr = self.transfer_data();
        acc_tr.balance += value;
        acc_storage.update_account(acc_tr)?;
        let tr_tr =
            tr_storage.create_transaction(self.name.clone(), TransactionAction::Add(value))?;
        self.balance += value;
        Ok(Transaction::from(tr_tr))
    }

    // task 2 part 2
    // decrements an account balance
    // errors: EmptyTransaction, Storage, NotEnoughMoney
    pub fn decr_balance<S: AccountStorage, T: TransactionStorage>(
        &mut self,
        value: usize,
        acc_storage: &mut S,
        tr_storage: &mut T,
    ) -> Result<Transaction, Error> {
        if value > self.balance {
            return Err(Error::NotEnoughMoney);
        }

        let mut raw = self.transfer_data();
        raw.balance -= value;
        acc_storage.update_account(raw)?;
        self.balance -= value;
        let tr_tr =
            tr_storage.create_transaction(self.name.clone(), TransactionAction::Withdraw(value))?;
        Ok(Transaction::from(tr_tr))
    }

    // task 3 make transactions from an one account to another
    // errors AccountNotExists Storage
    pub fn make_transaction<S: AccountStorage, T: TransactionStorage>(
        &mut self,
        value: usize,
        to: &mut Account,
        fee_amount: Option<usize>,
        acc_storage: &mut S,
        tr_storage: &mut T,
    ) -> Result<Transaction, Error> {
        let def_fee = 0;
        if value == 0 {
            Err(Error::EmptyTransaction)
        } else if value + fee_amount.unwrap_or(def_fee) > self.balance {
            Err(Error::NotEnoughMoney)
        } else {
            // create transaction
            let tr = tr_storage.create_transaction(
                self.name.clone(),
                TransactionAction::Transfer {
                    to: to.name.clone(),
                    value,
                    fee: fee_amount.unwrap_or(def_fee),
                },
            )?;

            // change sender
            self.balance -= value + fee_amount.unwrap_or(def_fee);
            self.trs.push(tr.id);
            acc_storage.update_account(self.transfer_data())?;

            // change receiver
            to.balance += value;
            to.trs.push(tr.id);
            acc_storage.update_account(to.transfer_data())?;

            // create fee transaction
            if fee_amount.unwrap_or(def_fee) > 0 {
                // increment fee acc
                let mut fee_acc = acc_storage.fee_account()?;
                fee_acc.balance += fee_amount.unwrap_or(def_fee);
                let tr = tr_storage.create_transaction(
                    acc_storage.fee_account()?.name,
                    TransactionAction::Add(fee_amount.unwrap_or(def_fee)),
                )?;
                fee_acc.trs.push(tr.id);
                acc_storage.update_account(fee_acc.clone())?;
            }

            Ok(Transaction::from(tr))
        }
    }

    pub fn transactions<T: TransactionStorage>(
        &self,
        tr_storage: &T,
    ) -> Result<Vec<Transaction>, Error> {
        Ok(self
            .trs
            .iter()
            .map(|id| tr_storage.transaction_by_id(*id))
            .filter(|tr| tr.is_ok())
            .map(|tr| Transaction::from(tr.unwrap()))
            .collect())
    }

    // restores account from transaction
    // errors: Storage
    pub fn from_transactions<S: AccountStorage>(
        account_name: String,
        trs: Vec<Transaction>,
        acc_storage: &mut S,
    ) -> Result<Account, Error> {
        let mut acc = Account { name: account_name, trs: trs.iter().map(|tr| tr.id).collect(), ..Default::default() };
        

        for tr in trs {
            match tr.action {
                TransactionAction::Registration => (),
                TransactionAction::Add(value) => acc.balance += value,
                TransactionAction::Withdraw(value) => acc.balance -= value,
                TransactionAction::Transfer { to, value, fee } => {
                    if to != acc.name {
                        acc.balance -= value + fee;
                    } else {
                        acc.balance += value
                    }
                }
            }
        }

        // try update account or recreate wit new data
        match acc_storage.update_account(AccountTransfer::from(&acc)) {
            Ok(acc) => Ok(Account {
                name: acc.name.clone(),
                balance: acc.balance,
                trs: acc.trs,
            }),
            Err(StorageError::AccountNotExists) => {
                let acc_t = acc_storage.create_account(AccountTransfer::from(&acc))?;
                Ok(Account::from(acc_t))
            }
            Err(err) => Err(Error::from(err)),
        }
    }

    // get transfer data
    fn transfer_data(&self) -> AccountTransfer {
        AccountTransfer {
            name: self.name.clone(),
            balance: self.balance,
            trs: self.trs.clone(),
        }
    }

    // task 10 get
    pub fn balance(&self) -> usize {
        self.balance
    }
}
