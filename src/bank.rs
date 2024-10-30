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

impl<A: AccountStorage, T: TransactionStorage> Bank<A, T> {
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

    pub fn inc_acc_balance(&mut self, acc: &mut Account, value: usize) -> Result<usize, AccError> {
        acc.inc_balance(value, &mut self.acc_storage, &mut self.tr_storage)
    }

    pub fn decr_acc_balance(&mut self, acc: &mut Account, value: usize) -> Result<usize, AccError> {
        acc.decr_balance(value, &mut self.acc_storage, &mut self.tr_storage)
    }

    pub fn make_transaction(
        &mut self,
        acc_from: &mut Account,
        acc_to: &mut Account,
        value: usize,
    ) -> Result<usize, AccError> {
        acc_from.make_transaction(
            value,
            acc_to,
            Some(self.tr_fee),
            &mut self.acc_storage,
            &mut self.tr_storage,
        )
    }

    pub fn restore_account_from_transactions(
        &mut self,
        account_name: String,
    ) -> Result<Account, AccError> {
        Account::restore_account_from_transactions(
            account_name,
            &mut self.acc_storage,
            &self.tr_storage,
        )
    }

    pub fn create_transaction(
        &mut self,
        account_name: String,
        action: TransactionAction,
    ) -> Result<Transaction, StorageError> {
        Ok(Transaction::from(
            self.tr_storage.create_transaction(account_name, action)?,
        ))
    }

    pub fn transactions(&self) -> Result<Vec<Transaction>, StorageError> {
        Ok(self
            .tr_storage
            .transactions()?
            .into_iter()
            .map(Transaction::from)
            .collect())
    }

    pub fn account_transactions(
        &self,
        account_name: String,
    ) -> Result<Vec<Transaction>, StorageError> {
        Ok(self
            .tr_storage
            .account_transactions(account_name)?
            .into_iter()
            .map(Transaction::from)
            .collect())
    }

    pub fn transaction_by_id(&self, id: usize) -> Result<Transaction, StorageError> {
        Ok(Transaction::from(self.tr_storage.transaction_by_id(id)?))
    }

    pub fn restore_accounts_from_bank_transactions(
        &mut self,
        bank: &Bank<A, T>,
    ) -> Result<(), AccError> {
        for acc in bank.accounts().unwrap() {
            Account::restore_account_from_transactions(
                acc.name.clone(),
                &mut self.acc_storage,
                &bank.tr_storage,
            )?;
        }
        Ok(())
    }
}
