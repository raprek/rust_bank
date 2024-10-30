use std::fmt::Display;

use crate::bank::storage::{
    AccountStorage, AccountTransfer, TransactionAction, TransactionStorage,
};

use super::storage::Error as StorageError;

#[derive(Debug)]
pub struct Account {
    pub balance: usize,
    pub name: String,
}

impl Display for Account {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Account: {}. Balance: {}", self.name, self.balance)
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum Error {
    Storage(String),
    AccountAlreadyExists,
    AccountNotExists,
    EmptyTransaction,
    NotEnoughMoney,
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
            name: name.clone(),
            balance: Default::default(),
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
    ) -> Result<usize, Error> {
        if value == 0 {
            return Err(Error::EmptyTransaction);
        }

        let mut acc_tr = self.transfer_data();
        acc_tr.balance += value;
        acc_storage.update_account(acc_tr)?;
        let tr_tr = tr_storage
            .create_transaction(self.name.clone(), TransactionAction::Increment(value))?;
        self.balance += value;
        Ok(tr_tr.id)
    }

    // task 2 part 2
    // decrements an account balance
    // errors: EmptyTransaction, Storage, NotEnoughMoney
    pub fn decr_balance<S: AccountStorage, T: TransactionStorage>(
        &mut self,
        value: usize,
        acc_storage: &mut S,
        tr_storage: &mut T,
    ) -> Result<usize, Error> {
        if value > self.balance {
            return Err(Error::NotEnoughMoney);
        }

        let mut raw = self.transfer_data();
        raw.balance -= value;
        acc_storage.update_account(raw)?;
        self.balance -= value;
        let tr_tr = tr_storage
            .create_transaction(self.name.clone(), TransactionAction::Decrement(value))?;
        Ok(tr_tr.id)
    }

    // task 3 make transactions from an one account to another
    pub fn make_transaction<S: AccountStorage, T: TransactionStorage>(
        &mut self,
        value: usize,
        to: &mut Account,
        fee_amount: Option<usize>,
        acc_storage: &mut S,
        tr_storage: &mut T,
    ) -> Result<usize, Error> {
        let def_fee = 0;
        if value == 0 {
            Err(Error::EmptyTransaction)
        } else if value + fee_amount.unwrap_or(def_fee) > self.balance {
            Err(Error::NotEnoughMoney)
        } else {
            let mut raw_self = self.transfer_data();
            raw_self.balance -= value + fee_amount.unwrap_or(def_fee);

            let mut raw_to = to.transfer_data();
            raw_to.balance += value;

            // increment balance of sender
            acc_storage.update_account(raw_self)?;
            let self_tr = tr_storage.create_transaction(
                self.name.clone(),
                TransactionAction::Decrement(value + fee_amount.unwrap_or(def_fee)),
            )?;
            self.balance -= value + fee_amount.unwrap_or(def_fee);

            // increment balance of receiver
            acc_storage.update_account(raw_to)?;
            tr_storage.create_transaction(to.name.clone(), TransactionAction::Increment(value))?;
            to.balance += value;

            // increment fee acc
            let mut fee_acc = acc_storage.fee_account()?;
            fee_acc.balance += fee_amount.unwrap_or(def_fee);
            acc_storage.update_account(fee_acc.clone())?;

            // create fee transaction
            if fee_amount.unwrap_or(def_fee) > 0 {
                tr_storage.create_transaction(
                    fee_acc.name,
                    TransactionAction::Increment(fee_amount.unwrap_or(def_fee)),
                )?;
            }

            Ok(self_tr.id)
        }
    }

    // restores account from transaction
    // errors: Storage
    pub fn restore_account_from_transactions<S: AccountStorage, T: TransactionStorage>(
        name: String,
        acc_storage: &mut S,
        tr_storage: &T,
    ) -> Result<Account, Error> {
        let trs = tr_storage.account_transactions(name.clone())?;
        let mut acc_t = AccountTransfer {
            name: name.clone(),
            balance: 0,
        };

        for tr in trs {
            match tr.action {
                TransactionAction::Registration => (),
                TransactionAction::Increment(amount) => acc_t.balance += amount,
                TransactionAction::Decrement(amount) => acc_t.balance -= amount,
            }
        }

        // try update account or recreate wit new data
        match acc_storage.update_account(acc_t.clone()) {
            Ok(acc) => Ok(Account {
                name: acc.name.clone(),
                balance: acc.balance,
            }),
            Err(StorageError::AccountNotExists) => {
                let acc_t = acc_storage.create_account(acc_t)?;
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
        }
    }

    // task 10 get
    pub fn balance(&self) -> usize {
        self.balance
    }
}
