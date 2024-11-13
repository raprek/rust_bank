use std::{collections::HashMap, fmt::Display};
use storage::{
    AccountStorage, AccountTransfer, Error as StorageError, TransactionAction, TransactionStorage,
    TransactionTransfer,
};

pub mod implements;
pub mod storage;

#[derive(Debug)]
pub struct Bank<A: AccountStorage, T: TransactionStorage> {
    acc_storage: A,
    tr_storage: T,
    tr_fee: usize,
}

#[derive(Debug, Default)]
pub struct Account {
    pub balance: usize,
    pub name: String,
    pub trs: Vec<usize>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Transaction {
    pub id: usize,
    pub action: TransactionAction,
    pub account_name: String,
}

#[derive(thiserror::Error, Debug, PartialEq, Eq)]
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

impl Display for Account {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Account: {}. Balance: {}", self.name, self.balance)
    }
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

impl From<Account> for AccountTransfer {
    fn from(value: Account) -> Self {
        AccountTransfer {
            name: value.name.clone(),
            balance: value.balance,
            trs: value.trs.clone(),
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

impl<A: AccountStorage + Default, T: TransactionStorage + Default> Bank<A, T> {
    pub fn new(acc_storage: A, tr_storage: T, tr_fee: Option<usize>) -> Self {
        Bank {
            acc_storage,
            tr_storage,
            tr_fee: tr_fee.unwrap_or(0),
        }
    }

    pub fn create_account(&mut self, account_name: String) -> Result<Account, Error> {
        let tr = self
            .tr_storage
            .create_transaction(account_name.clone(), TransactionAction::Registration)?;

        let mut acc = AccountTransfer::new(account_name.clone(), None);
        acc.trs.push(tr.id);
        self.acc_storage.create_account(acc)?;

        Ok(Account {
            name: account_name,
            balance: Default::default(),
            trs: vec![tr.id],
        })
    }

    pub fn accounts(&self) -> Result<Vec<Account>, Error> {
        let accs = self
            .acc_storage
            .accounts()?
            .into_iter()
            .map(Account::from)
            .collect::<Vec<Account>>();
        Ok(accs)
    }

    pub fn account(&self, account_name: String) -> Result<Account, Error> {
        Ok(Account::from(self.acc_storage.get_account(account_name)?))
    }

    pub fn inc_acc_balance(&mut self, account_name: String, value: usize) -> Result<usize, Error> {
        if value == 0 {
            return Err(Error::EmptyTransaction);
        }
        let tr = self
            .tr_storage
            .create_transaction(account_name.clone(), TransactionAction::Add(value))?;

        let mut acc_tr = AccountTransfer::from(self.account(account_name.clone())?);
        acc_tr.balance += value;
        acc_tr.trs.push(tr.id);

        self.acc_storage.update_account(acc_tr)?;
        Ok(tr.id)
    }

    pub fn decr_acc_balance(&mut self, account_name: String, value: usize) -> Result<usize, Error> {
        let mut acc = self.account(account_name.clone())?;
        if value > acc.balance {
            return Err(Error::NotEnoughMoney);
        } else if value == 0 {
            return Err(Error::EmptyTransaction);
        }

        let tr = self
            .tr_storage
            .create_transaction(account_name, TransactionAction::Withdraw(value))?;

        acc.balance -= value;
        acc.trs.push(tr.id);
        self.acc_storage
            .update_account(AccountTransfer::from(acc))?;

        Ok(tr.id)
    }

    pub fn make_transaction(
        &mut self,
        account_name_from: String,
        account_name_to: String,
        value: usize,
    ) -> Result<usize, Error> {
        let mut acc_from = self.account(account_name_from.clone())?;
        if value == 0 {
            Err(Error::EmptyTransaction)
        } else if value + self.tr_fee > acc_from.balance {
            Err(Error::NotEnoughMoney)
        } else {
            // create transaction
            let tr = self.tr_storage.create_transaction(
                account_name_from.clone(),
                TransactionAction::Transfer {
                    to: account_name_to.clone(),
                    value,
                    fee: self.tr_fee,
                },
            )?;

            // change sender
            acc_from.balance -= value + self.tr_fee;
            acc_from.trs.push(tr.id);
            self.acc_storage
                .update_account(AccountTransfer::from(acc_from))?;

            // change receiver
            let mut acc_to = self.account(account_name_to.clone())?;
            acc_to.balance += value;
            acc_to.trs.push(tr.id);
            self.acc_storage
                .update_account(AccountTransfer::from(acc_to))?;

            // create fee transaction
            if self.tr_fee > 0 {
                // increment fee acc
                let mut fee_acc = self.acc_storage.fee_account()?;
                fee_acc.balance += self.tr_fee;
                let tr = self.tr_storage.create_transaction(
                    self.acc_storage.fee_account()?.name,
                    TransactionAction::Add(self.tr_fee),
                )?;
                fee_acc.trs.push(tr.id);
                self.acc_storage.update_account(fee_acc.clone())?;
            }

            Ok(tr.id)
        }
    }

    fn restore_account_from_transactions(
        &mut self,
        account_name: String,
        trs: Vec<Transaction>,
    ) -> Result<Account, Error> {
        let mut acc = Account {
            name: account_name,
            trs: trs.iter().map(|tr| tr.id).collect(),
            ..Default::default()
        };

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
        match self.acc_storage.update_account(AccountTransfer::from(&acc)) {
            Ok(acc) => Ok(Account {
                name: acc.name.clone(),
                balance: acc.balance,
                trs: acc.trs,
            }),
            Err(StorageError::AccountNotExists) => {
                let acc_t = self
                    .acc_storage
                    .create_account(AccountTransfer::from(&acc))?;
                Ok(Account::from(acc_t))
            }
            Err(err) => Err(Error::from(err)),
        }
    }

    pub fn transactions(&self) -> Result<Vec<Transaction>, Error> {
        Ok(self
            .tr_storage
            .transactions()?
            .into_iter()
            .map(Transaction::from)
            .collect())
    }

    pub fn account_transactions(
        &mut self,
        account_name: String,
    ) -> Result<Vec<Transaction>, Error> {
        let acc = self.account(account_name.clone())?;
        Ok(acc
            .trs
            .iter()
            .map(|id| self.tr_storage.transaction_by_id(*id))
            .filter(|tr| tr.is_ok())
            .map(|tr| Transaction::from(tr.unwrap()))
            .collect())
    }

    pub fn transaction_by_id(&self, id: usize) -> Result<Transaction, Error> {
        Ok(Transaction::from(self.tr_storage.transaction_by_id(id)?))
    }

    pub fn restore_bank_from_transactions(
        trs: Vec<Transaction>,
        tr_fee: Option<usize>,
    ) -> Result<Bank<A, T>, Error> {
        let mut bank = Bank::new(A::default(), T::default(), tr_fee);
        let mut restore_map: HashMap<String, Vec<Transaction>> = HashMap::new();

        for tr in trs {
            if let TransactionAction::Transfer {
                to,
                value: _,
                fee: _,
            } = tr.action.clone()
            {
                match restore_map.entry(to.clone()) {
                    std::collections::hash_map::Entry::Occupied(mut occupied_entry) => {
                        occupied_entry.get_mut().push(tr.clone());
                    }
                    std::collections::hash_map::Entry::Vacant(vacant_entry) => {
                        vacant_entry.insert(vec![tr.clone()]);
                    }
                }
            }
            match restore_map.entry(tr.account_name.clone()) {
                std::collections::hash_map::Entry::Occupied(mut occupied_entry) => {
                    occupied_entry.get_mut().push(tr.clone());
                }
                std::collections::hash_map::Entry::Vacant(vacant_entry) => {
                    vacant_entry.insert(vec![tr.clone()]);
                }
            }

            let _ = bank
                .tr_storage
                .create_transaction(tr.account_name.clone(), tr.action.clone());
        }

        for (acc_name, trs) in restore_map.into_iter() {
            let _ = bank.restore_account_from_transactions(acc_name, trs)?;
        }

        Ok(bank)
    }

    pub fn account_balance(&self, account_name: String) -> Result<usize, Error> {
        let acc = self.account(account_name)?;
        Ok(acc.balance)
    }
}
