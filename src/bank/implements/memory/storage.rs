use crate::bank::traits::storage::{
    AccountStorage, AccountTransfer, GetTransactionByIdError, GetTransactionError,
    GetTransactionsError, Storage, StorageCreateAccountError, StorageCreateTransactionError,
    StorageGetAccountError, StorageUpdateAccountError, TransactionAction, TransactionStorage,
    TransactionTransfer,
};
use std::{borrow::Borrow, cell::RefCell, collections::HashMap, path::Iter, rc::Rc};

pub struct MemAccountStorage {
    storage: HashMap<String, AccountTransfer>,
    // history_storage:
}

#[derive(Clone, Copy)]
pub struct MemTransactionStorageItem {
    pub id: usize,
    pub action: TransactionAction,
    pub amount: usize,
}

pub struct MemTransactionStorage {
    storage: HashMap<String, Vec<MemTransactionStorageItem>>,
    last_tr_id: usize,
}

impl MemAccountStorage {
    fn new() -> Self {
        MemAccountStorage {
            storage: Default::default(),
        }
    }
}

impl MemTransactionStorage {
    fn new() -> Self {
        MemTransactionStorage {
            storage: Default::default(),
            last_tr_id: 0,
        }
    }
}

impl From<MemTransactionStorageItem> for TransactionTransfer {
    fn from(value: MemTransactionStorageItem) -> Self {
        TransactionTransfer {
            id: value.id,
            amount: value.amount,
            action: value.action,
            account_name: String::new(),
        }
    }
}

impl AccountStorage for MemAccountStorage {
    fn create_account(
        &mut self,
        raw_data: AccountTransfer,
    ) -> Result<&AccountTransfer, StorageCreateAccountError> {
        match self.storage.entry(raw_data.name.clone()) {
            std::collections::hash_map::Entry::Occupied(_) => {
                Err(StorageCreateAccountError::AccountAlreadyExists)
            }
            std::collections::hash_map::Entry::Vacant(vacant) => Ok(vacant.insert(raw_data)),
        }
    }

    fn get_account(&self, name: String) -> Result<&AccountTransfer, StorageGetAccountError> {
        match self.storage.get(&name) {
            Some(acc) => Ok(acc),
            None => Err(StorageGetAccountError::AccountNotExists),
        }
    }

    fn update_account(
        &mut self,
        raw_data: AccountTransfer,
    ) -> Result<&AccountTransfer, StorageUpdateAccountError> {
        let key = raw_data.name.clone();
        match self.storage.entry(key.clone()) {
            std::collections::hash_map::Entry::Occupied(mut occ) => {
                occ.insert(raw_data);
            }
            std::collections::hash_map::Entry::Vacant(_) => {
                return Err(StorageUpdateAccountError::AccountNotExists)
            }
        }

        Ok(self.storage.get(&key).unwrap())
    }
}

impl TransactionStorage for MemTransactionStorage {
    fn create_transaction(
        &mut self,
        account_name: String,
        amount: usize,
        action: TransactionAction,
    ) -> Result<TransactionTransfer, StorageCreateTransactionError> {
        self.last_tr_id += 1;
        let item = MemTransactionStorageItem {
            id: self.last_tr_id,
            amount: amount,
            action: action,
        };
        match self.storage.entry(account_name.clone()) {
            std::collections::hash_map::Entry::Occupied(mut occupied_entry) => {
                occupied_entry.get_mut().push(item);
            }
            std::collections::hash_map::Entry::Vacant(vacant_entry) => {
                vacant_entry.insert(vec![item]);
            }
        }
        Ok(TransactionTransfer::from(item))
    }

    fn transactions(&self) -> Result<Vec<TransactionTransfer>, GetTransactionsError> {
        let mut transactions = Vec::new();
        for (name, trs) in self.storage.iter() {
            for tr in trs.iter() {
                let mut tt = TransactionTransfer::from(*tr);
                tt.account_name = name.clone();
                transactions.push(tt);
            }
        }
        Ok(transactions)
    }

    // O(n); n - number of an account transactions
    fn account_transactions(
        &self,
        account_name: String,
    ) -> Result<Vec<TransactionTransfer>, GetTransactionError> {
        let mut transactions = Vec::new();
        if let Some(trs) = self.storage.get(&account_name) {
            for tr in trs.iter() {
                let mut tt = TransactionTransfer::from(*tr);
                tt.account_name = account_name.clone();
                transactions.push(tt);
            }
            Ok(transactions)
        } else {
            Err(GetTransactionError::AccountNotExists)
        }
    }

    // O(n); n - number of transactions
    fn get_transaction_by_id(
        &self,
        id: usize,
    ) -> Result<TransactionTransfer, GetTransactionByIdError> {
        match self.transactions() {
            Ok(trs) => match trs.into_iter().filter(|x| x.id == id).last() {
                Some(tr) => Ok(tr),
                None => Err(GetTransactionByIdError::NotFound),
            },
            Err(err) => match err {
                GetTransactionsError::StorageError(err) => {
                    Err(GetTransactionByIdError::StorageError(err))
                }
            },
        }
    }
}

#[cfg(test)]
mod tests {

    use crate::bank::traits::account::Account;

    use super::*;

    #[test]
    fn test_storage_get_account() {
        let mut storage = MemAccountStorage::new();
        let test_name = "test".to_string();

        // test empty get
        assert_eq!(storage.get_account(test_name.clone()).is_err(), true);

        // test success insert
        let raw = AccountTransfer {
            name: test_name.clone(),
            balance: 0,
        };
        assert_eq!(storage.create_account(raw).is_ok(), true);

        let result = storage.get_account(test_name.clone());
        assert_eq!(result.is_ok(), true);
        assert_eq!(result.unwrap(), storage.storage.get(&test_name).unwrap());
    }

    #[test]
    fn test_storage_create_account() {
        let mut storage = MemAccountStorage::new();
        let test_name = "test".to_string();

        // test add new acc (not existed early)
        let mut raw = AccountTransfer {
            name: test_name.clone(),
            balance: 0,
        };
        assert_eq!(storage.create_account(raw).is_ok(), true);

        // test create acc with same name
        raw = AccountTransfer {
            name: test_name.clone(),
            balance: 0,
        };
        let result = storage.create_account(raw);
        assert_eq!(result.is_err(), true);
        assert_eq!(
            result.err().unwrap(),
            StorageCreateAccountError::AccountAlreadyExists
        );
    }

    #[test]
    fn test_storage_update_account() {
        let mut storage = MemAccountStorage::new();
        let test_name = "test".to_string();

        // updates non existed account
        let raw = AccountTransfer {
            name: "not_exist".to_string(),
            balance: 0,
        };
        let mut result = storage.update_account(raw);
        assert_eq!(result.is_err(), true);
        assert_eq!(
            result.err().unwrap(),
            StorageUpdateAccountError::AccountNotExists
        );

        // test add new acc (not existed early)
        let raw = AccountTransfer {
            name: test_name.clone(),
            balance: 0,
        };
        let acc = storage.create_account(raw).unwrap();

        let to_update = AccountTransfer {
            name: acc.name.clone(),
            balance: 123,
        };
        let res = storage.update_account(to_update);
        assert_eq!(res.is_ok(), true);
        assert_eq!(res.unwrap().balance, 123);
    }

    #[test]
    fn test_storage_create_transaction() {
        let mut storage = MemTransactionStorage::new();

        let account_name = "test".to_string();

        let mut res = storage
            .create_transaction(account_name.clone(), 13, TransactionAction::Registration)
            .unwrap();
        assert_eq!(res.id, 1);
        assert_eq!(res.amount, 13);
        assert_eq!(storage.storage.get(&account_name).unwrap().len(), 1);

        res = storage
            .create_transaction(account_name.clone(), 14, TransactionAction::Registration)
            .unwrap();
        assert_eq!(res.id, 2);
        assert_eq!(res.amount, 14);
        assert_eq!(storage.storage.get(&account_name).unwrap().len(), 2)
    }

    #[test]
    fn test_storage_transactions() {
        let mut storage = MemTransactionStorage::new();
        storage
            .create_transaction("test_1".to_owned(), 13, TransactionAction::Registration)
            .unwrap();
        storage
            .create_transaction("test_1".to_owned(), 14, TransactionAction::Registration)
            .unwrap();
        storage
            .create_transaction("test_2".to_owned(), 13, TransactionAction::Increment)
            .unwrap();
        storage
            .create_transaction("test_3".to_owned(), 13, TransactionAction::Decrement)
            .unwrap();

        let transactions = storage.transactions().unwrap();
        assert_eq!(transactions.len(), 4);
        assert_eq!(
            transactions
                .iter()
                .filter(|x| x.account_name == "test_1")
                .count(),
            2
        );
        assert_eq!(
            transactions
                .iter()
                .filter(|x| x.account_name == "test_2")
                .count(),
            1
        );
        assert_eq!(
            transactions
                .iter()
                .filter(|x| x.account_name == "test_3")
                .count(),
            1
        );
    }

    #[test]
    fn test_storage_account_transactions() {
        let mut storage = MemTransactionStorage::new();
        storage
            .create_transaction("test_1".to_owned(), 13, TransactionAction::Registration)
            .unwrap();
        storage
            .create_transaction("test_1".to_owned(), 14, TransactionAction::Registration)
            .unwrap();
        storage
            .create_transaction("test_2".to_owned(), 13, TransactionAction::Increment)
            .unwrap();
        storage
            .create_transaction("test_3".to_owned(), 13, TransactionAction::Decrement)
            .unwrap();
        storage
            .create_transaction("test_3".to_owned(), 11, TransactionAction::Decrement)
            .unwrap();

        assert_eq!(
            storage
                .account_transactions("test_1".to_owned())
                .unwrap()
                .len(),
            2
        );
        assert_eq!(
            storage
                .account_transactions("test_2".to_owned())
                .unwrap()
                .len(),
            1
        );
        assert_eq!(
            storage
                .account_transactions("test_3".to_owned())
                .unwrap()
                .len(),
            2
        );
    }

    #[test]
    fn test_storage_get_transaction_by_id() {
        let mut storage = MemTransactionStorage::new();
        storage
            .create_transaction("test_1".to_owned(), 13, TransactionAction::Registration)
            .unwrap();
        storage
            .create_transaction("test_2".to_owned(), 14, TransactionAction::Registration)
            .unwrap();
        storage
            .create_transaction("test_3".to_owned(), 15, TransactionAction::Increment)
            .unwrap();

        assert_eq!(storage.get_transaction_by_id(1).unwrap().id, 1);
        assert_eq!(storage.get_transaction_by_id(1).unwrap().amount, 13);

        assert_eq!(storage.get_transaction_by_id(2).unwrap().id, 2);
        assert_eq!(storage.get_transaction_by_id(2).unwrap().amount, 14);

        assert_eq!(storage.get_transaction_by_id(3).unwrap().id, 3);
        assert_eq!(storage.get_transaction_by_id(3).unwrap().amount, 15);

        assert_eq!(storage.get_transaction_by_id(4).is_err(), true);
    }

    #[test]
    fn test_account_new() {
        let mut storage = Storage::new(MemAccountStorage::new(), MemTransactionStorage::new());
        let target_name = "test".to_string();

        // test create account with new name
        let mut acc = Account::new(target_name.clone(), storage.clone());
        assert_eq!(acc.is_ok(), true);

        // // test error to create acc with same name
        // acc = Account::new(target_name.clone(), storage);
        // assert_eq!(acc.is_err(), true);
    }
}
