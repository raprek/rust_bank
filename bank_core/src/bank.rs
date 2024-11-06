use std::collections::HashMap;

use account::{Account, Error as AccError};
use storage::{AccountStorage, Error as StorageError, TransactionAction, TransactionStorage};
use transactions::Transaction;

pub mod account;
pub mod implements;
pub mod storage;
pub mod transactions;

pub struct Bank<A: AccountStorage, T: TransactionStorage> {
    acc_storage: A,
    tr_storage: T,
    tr_fee: usize,
}

impl<A: AccountStorage + Default, T: TransactionStorage + Default> Bank<A, T> {
    pub fn new(acc_storage: A, tr_storage: T, tr_fee: Option<usize>) -> Self {
        Bank {
            acc_storage,
            tr_storage,
            tr_fee: tr_fee.unwrap_or(0),
        }
    }
    pub fn accounts(&self) -> Result<Vec<Account>, AccError> {
        let accs = self
            .acc_storage
            .accounts()?
            .into_iter()
            .map(Account::from)
            .collect::<Vec<Account>>();
        Ok(accs)
    }

    pub fn create_account(&mut self, account_name: String) -> Result<Account, AccError> {
        Account::new(account_name, &mut self.acc_storage, &mut self.tr_storage)
    }

    pub fn get_acc(&mut self, account_name: String) -> Result<Account, AccError> {
        Ok(
            Account::from(self.acc_storage.get_account(account_name)?)
        )
    }

    pub fn inc_acc_balance(&mut self, acc: &mut Account, value: usize) -> Result<usize, AccError> {
        Ok(acc
            .inc_balance(value, &mut self.acc_storage, &mut self.tr_storage)?
            .id)
    }

    pub fn decr_acc_balance(&mut self, acc: &mut Account, value: usize) -> Result<usize, AccError> {
        Ok(acc
            .decr_balance(value, &mut self.acc_storage, &mut self.tr_storage)?
            .id)
    }

    pub fn make_transaction(
        &mut self,
        acc_from: &mut Account,
        acc_to: &mut Account,
        value: usize,
    ) -> Result<usize, AccError> {
        Ok(acc_from
            .make_transaction(
                value,
                acc_to,
                Some(self.tr_fee),
                &mut self.acc_storage,
                &mut self.tr_storage,
            )?
            .id)
    }

    pub fn restore_account_from_transactions(
        &mut self,
        account_name: String,
    ) -> Result<Account, AccError> {
        let acc = Account::from(self.acc_storage.get_account(account_name)?);
        Account::from_transactions(
            acc.name.clone(),
            acc.transactions(&self.tr_storage)?,
            &mut self.acc_storage,
        )
    }

    pub fn transactions(&self) -> Result<Vec<Transaction>, StorageError> {
        Ok(self
            .tr_storage
            .transactions()?
            .into_iter()
            .map(Transaction::from)
            .collect())
    }

    pub fn account_transactions(&self, account_name: String) -> Result<Vec<Transaction>, AccError> {
        let acc = Account::from(self.acc_storage.get_account(account_name)?);
        acc.transactions(&self.tr_storage)
    }

    pub fn transaction_by_id(&self, id: usize) -> Result<Transaction, StorageError> {
        Ok(Transaction::from(self.tr_storage.transaction_by_id(id)?))
    }

    pub fn restore_bank_from_transactions(
        trs: Vec<Transaction>,
        tr_fee: Option<usize>,
    ) -> Result<Bank<A, T>, AccError> {
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
            let _ = Account::from_transactions(acc_name, trs, &mut bank.acc_storage)?;
        }

        Ok(bank)
    }
}
