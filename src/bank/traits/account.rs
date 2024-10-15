use std::{cell::RefCell, rc::Rc};

use crate::bank::traits::storage::{
    AccountStorage, AccountTransfer, Storage, StorageCreateTransactionError,
    StorageUpdateAccountError, TransactionAction, TransactionStorage,
};

use super::storage::StorageCreateAccountError;

#[derive(Debug)]
pub struct Account<A: AccountStorage, T: TransactionStorage> {
    balance: usize,
    pub name: String,
    storage: Rc<RefCell<Storage<A, T>>>,
}

#[derive(Debug)]
pub enum CreateAccountError {
    TransactionStorage(StorageCreateTransactionError),
    AccountStorage(StorageCreateAccountError),
}

#[derive(Debug)]
pub enum IncBalanceError {
    ZeroInc,
    NegativeBalance,
    TransactionStorage(StorageCreateTransactionError),
    AccountStorage(StorageUpdateAccountError),
}

#[derive(Debug)]
pub enum DecBalanceError {
    ZeroDec,
    AccountNotExisted,
    TransactionStorage(StorageCreateTransactionError),
    AccountStorage(StorageUpdateAccountError),
}

#[derive(Debug)]
pub enum TransferError {
    ZeroTransfer,
    NotEnoughBalance,
    TransactionStorage(StorageCreateTransactionError),
    AccountStorage(StorageUpdateAccountError),
}

impl From<StorageCreateTransactionError> for IncBalanceError {
    fn from(value: StorageCreateTransactionError) -> Self {
        IncBalanceError::TransactionStorage(value)
    }
}

impl From<StorageUpdateAccountError> for IncBalanceError {
    fn from(value: StorageUpdateAccountError) -> Self {
        IncBalanceError::AccountStorage(value)
    }
}

impl From<StorageCreateTransactionError> for DecBalanceError {
    fn from(value: StorageCreateTransactionError) -> Self {
        DecBalanceError::TransactionStorage(value)
    }
}

impl From<StorageCreateTransactionError> for TransferError {
    fn from(value: StorageCreateTransactionError) -> Self {
        TransferError::TransactionStorage(value)
    }
}

impl From<StorageUpdateAccountError> for TransferError {
    fn from(value: StorageUpdateAccountError) -> Self {
        TransferError::AccountStorage(value)
    }
}

impl<'s, S: AccountStorage, T: TransactionStorage> Account<S, T> {
    // task 1 create an account
    pub fn new(
        name: String,
        storage: Rc<RefCell<Storage<S, T>>>,
    ) -> Result<Account<S, T>, CreateAccountError> {

        
        match storage.clone().borrow().acc_storage.clone().borrow_mut().create_account(AccountTransfer {
            name,
            balance: Default::default(),
        }) {
            Ok(raw) => {
                match storage.clone().borrow().tr_storage.clone().borrow_mut().create_transaction(
                    raw.name.clone(),
                    0,
                    TransactionAction::Registration,
                ) {
                    Ok(_) => Ok(Account {
                        name: raw.name.clone(),
                        balance: raw.balance,
                        storage: storage.clone(),
                    }),
                    Err(err) => Err(CreateAccountError::TransactionStorage(err)),
                }
            }
            Err(err) => {
                Err(CreateAccountError::AccountStorage(err.clone()))
                //
            }
        }
    }

    // task 2 part 1
    pub fn inc_balance(&mut self, value: usize) -> Result<(), IncBalanceError> {
        if value == 0 {
            Err(IncBalanceError::ZeroInc)
        } else {
            let mut raw = self.raw_data();
            raw.balance += value;

            self.storage
                .clone()
                .borrow_mut()
                .acc_storage
                .clone()
                .borrow_mut()
                .update_account(raw)?;
            self.balance += value;
            self.storage
                .clone()
                .borrow_mut()
                .tr_storage
                .clone()
                .borrow_mut()
                .create_transaction(self.name.clone(), value, TransactionAction::Increment)?;
            Ok(())
        }
    }

    // task 2 part 2
    pub fn decr_balance(&mut self, value: usize) -> Result<(), IncBalanceError> {
        if value > self.balance {
            Err(IncBalanceError::NegativeBalance)
        } else {
            let mut raw = self.raw_data();
            raw.balance -= value;
            self.storage
                .clone()
                .borrow_mut()
                .acc_storage
                .update_account(raw)?;
            self.balance -= value;
            self.storage
                .clone()
                .borrow_mut()
                .tr_storage
                .create_transaction(self.name.clone(), value, TransactionAction::Decrement)?;
            Ok(())
        }
    }

    // task 3 make transactions from an one account to another
    fn make_transaction(
        &mut self,
        value: usize,
        to: &mut Account<S, T>,
    ) -> Result<(), TransferError> {
        if value == 0 {
            Err(TransferError::ZeroTransfer)
        } else if value > self.balance {
            Err(TransferError::NotEnoughBalance)
        } else {
            let mut raw_self = self.raw_data();
            raw_self.balance -= value;

            let mut raw_to = to.raw_data();
            raw_to.balance += value;

            // increment balance of sender
            self.storage
                .clone()
                .borrow_mut()
                .acc_storage
                .update_account(raw_self)?;
            self.storage
                .clone()
                .borrow_mut()
                .tr_storage
                .create_transaction(self.name.clone(), value, TransactionAction::Decrement)?;
            self.balance -= value;

            // increment balance of receiver
            self.storage
                .clone()
                .borrow_mut()
                .acc_storage
                .update_account(raw_to)?;
            self.storage
                .clone()
                .borrow_mut()
                .tr_storage
                .create_transaction(to.name.clone(), value, TransactionAction::Increment)?;
            to.balance += value;

            Ok(())
        }
    }

    // todo move to generic

    fn raw_data(&self) -> AccountTransfer {
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
