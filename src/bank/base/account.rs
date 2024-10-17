use std::{fmt::Display, rc::Rc};

use crate::bank::base::storage::{
    AccountStorage, AccountTransfer, Storage, StorageCreateTransactionError,
    StorageUpdateAccountError, TransactionAction, TransactionStorage,
};

use super::storage::{FeeAccountError, GetTransactionError, StorageCreateAccountError};

#[derive(Debug)]
pub struct Account<A: AccountStorage, T: TransactionStorage> {
    balance: usize,
    pub name: String,
    storage: Rc<Storage<A, T>>,
}

#[derive(Debug)]
pub enum CreateAccountError {
    TransactionStorage(StorageCreateTransactionError),
    AccountStorage(StorageCreateAccountError),
}

#[derive(Debug, PartialEq)]
pub enum IncBalanceError {
    ZeroInc,
    TransactionStorage(StorageCreateTransactionError),
    AccountStorage(StorageUpdateAccountError),
}

#[derive(Debug)]
pub enum DecBalanceError {
    ZeroDec,
    NotEnoughMoney,
    TransactionStorage(StorageCreateTransactionError),
    AccountStorage(StorageUpdateAccountError),
}

#[derive(Debug)]
pub enum TransferError {
    ZeroTransfer,
    NotEnoughBalance,
    CreateTransaction(StorageCreateTransactionError),
    UpdateAccount(StorageUpdateAccountError),
    GetFeeAccount(FeeAccountError),
}

#[derive(Debug)]
pub enum RestoreAccountError {
    StorageAccount(String),
    StorageCreateAccount(StorageCreateAccountError),
    GetTransaction(GetTransactionError),
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

impl From<StorageUpdateAccountError> for DecBalanceError {
    fn from(value: StorageUpdateAccountError) -> Self {
        DecBalanceError::AccountStorage(value)
    }
}

impl From<StorageCreateTransactionError> for DecBalanceError {
    fn from(value: StorageCreateTransactionError) -> Self {
        DecBalanceError::TransactionStorage(value)
    }
}

impl From<StorageCreateTransactionError> for TransferError {
    fn from(value: StorageCreateTransactionError) -> Self {
        TransferError::CreateTransaction(value)
    }
}

impl From<StorageUpdateAccountError> for TransferError {
    fn from(value: StorageUpdateAccountError) -> Self {
        TransferError::UpdateAccount(value)
    }
}

impl From<FeeAccountError> for TransferError {
    fn from(value: FeeAccountError) -> Self {
        TransferError::GetFeeAccount(value)
    }
}

impl From<StorageCreateAccountError> for RestoreAccountError {
    fn from(value: StorageCreateAccountError) -> Self {
        RestoreAccountError::StorageCreateAccount(value)
    }
}

impl<S: AccountStorage, T: TransactionStorage> Display for Account<S, T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Account: {}. Balance: {}", self.name, self.balance)
    }
}

impl<S: AccountStorage, T: TransactionStorage> Account<S, T> {
    // task 1 create an account
    pub fn new(
        name: String,
        storage: Rc<Storage<S, T>>,
    ) -> Result<Account<S, T>, CreateAccountError> {
        match storage
            .acc_storage
            .borrow_mut()
            .create_account(AccountTransfer {
                name,
                balance: Default::default(),
            }) {
            Ok(raw) => {
                match storage.tr_storage.borrow_mut().create_transaction(
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
            Err(err) => Err(CreateAccountError::AccountStorage(err.clone())),
        }
    }

    // task 2 part 1
    pub fn inc_balance(&mut self, value: usize) -> Result<usize, IncBalanceError> {
        if value == 0 {
            Err(IncBalanceError::ZeroInc)
        } else {
            let mut raw = self.transfer_data();
            raw.balance += value;

            self.storage.acc_storage.borrow_mut().update_account(raw)?;
            self.balance += value;
            Ok(self
                .storage
                .tr_storage
                .borrow_mut()
                .create_transaction(self.name.clone(), value, TransactionAction::Increment)?
                .id)
        }
    }

    // task 2 part 2
    pub fn decr_balance(&mut self, value: usize) -> Result<usize, DecBalanceError> {
        if value > self.balance {
            Err(DecBalanceError::NotEnoughMoney)
        } else {
            let mut raw = self.transfer_data();
            raw.balance -= value;
            self.storage.acc_storage.borrow_mut().update_account(raw)?;
            self.balance -= value;
            Ok(self
                .storage
                .tr_storage
                .borrow_mut()
                .create_transaction(self.name.clone(), value, TransactionAction::Decrement)?
                .id)
        }
    }

    // task 3 make transactions from an one account to another
    pub fn make_transaction(
        &mut self,
        value: usize,
        to: &mut Account<S, T>,
        fee_amount: Option<usize>,
    ) -> Result<usize, TransferError> {
        let def_fee = 0;
        if value == 0 {
            Err(TransferError::ZeroTransfer)
        } else if value + fee_amount.unwrap_or(def_fee) > self.balance {
            Err(TransferError::NotEnoughBalance)
        } else {
            let mut raw_self = self.transfer_data();
            raw_self.balance -= value + fee_amount.unwrap_or(def_fee);

            let mut raw_to = to.transfer_data();
            raw_to.balance += value;

            // increment balance of sender
            self.storage
                .acc_storage
                .borrow_mut()
                .update_account(raw_self)?;
            let self_tr = self.storage.tr_storage.borrow_mut().create_transaction(
                self.name.clone(),
                value + fee_amount.unwrap_or(def_fee),
                TransactionAction::Decrement,
            )?;
            self.balance -= value + fee_amount.unwrap_or(def_fee);

            // increment balance of receiver
            self.storage
                .acc_storage
                .borrow_mut()
                .update_account(raw_to)?;
            self.storage.tr_storage.borrow_mut().create_transaction(
                to.name.clone(),
                value,
                TransactionAction::Increment,
            )?;
            to.balance += value;

            // increment fee acc
            let mut fee_acc = self.storage.acc_storage.borrow_mut().fee_account()?;
            fee_acc.balance += fee_amount.unwrap_or(def_fee);
            self.storage
                .acc_storage
                .borrow_mut()
                .update_account(fee_acc)?;

            Ok(self_tr.id)
        }
    }

    pub fn restore_account_from_transactions(
        name: String,
        storage: Rc<Storage<S, T>>,
    ) -> Result<Account<S, T>, RestoreAccountError> {
        let trs = storage
            .tr_storage
            .borrow()
            .account_transactions(name.clone());
        match trs {
            Ok(trs) => {
                let mut acc_t = AccountTransfer {
                    name: name.clone(),
                    balance: 0,
                };
                for tr in trs {
                    match tr.action {
                        TransactionAction::Registration => (),
                        TransactionAction::Increment => acc_t.balance += tr.amount,
                        TransactionAction::Decrement => acc_t.balance -= tr.amount,
                    }
                }

                let mut acc_storage_ref = storage.acc_storage.borrow_mut();
                // try update account or recreate wit new data
                match acc_storage_ref.update_account(acc_t.clone()) {
                    Ok(acc) => Ok(Account {
                        name: acc.name.clone(),
                        balance: acc.balance,
                        storage: storage.clone(),
                    }),
                    Err(err) => match err {
                        StorageUpdateAccountError::StorageError(e) => {
                            Err(RestoreAccountError::StorageAccount(e))
                        }

                        // creates account if not existed
                        StorageUpdateAccountError::AccountNotExists => {
                            acc_t = storage.acc_storage.borrow_mut().create_account(acc_t)?;
                            Ok(Account {
                                name: acc_t.name,
                                balance: acc_t.balance,
                                storage: storage.clone(),
                            })
                        }
                    },
                }
            }
            Err(err) => Err(RestoreAccountError::GetTransaction(err)),
        }
    }

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
