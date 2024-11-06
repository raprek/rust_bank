use crate::bank::storage::{
    AccountStorage, AccountTransfer, Error, TransactionAction, TransactionStorage,
    TransactionTransfer,
};
use std::collections::HashMap;

#[derive(Default)]
pub struct MemAccountStorage {
    storage: HashMap<String, AccountTransfer>,
    // name reserved for bank fees account
    fee_acc_name: String,
}

#[derive(Clone, Default)]
pub struct MemTransactionStorageItem {
    pub id: usize,
    pub action: TransactionAction,
    pub account_name: String,
}

pub struct MemTransactionStorage {
    storage: Vec<MemTransactionStorageItem>,
    last_tr_id: usize,
}

impl MemAccountStorage {
    pub fn new() -> Result<Self, Error> {
        let fee_acc_name = "fee_acc".to_string();
        let mut s = MemAccountStorage {
            storage: Default::default(),
            fee_acc_name: fee_acc_name.clone(),
        };

        let _ = s.create_account(AccountTransfer {
            name: fee_acc_name,
            balance: 0,
            trs: Default::default(),
        })?;
        Ok(s)
    }
}

impl MemTransactionStorage {
    pub fn new() -> Self {
        MemTransactionStorage {
            storage: Default::default(),
            last_tr_id: 0,
        }
    }
}

impl Default for MemTransactionStorage {
    fn default() -> Self {
        Self::new()
    }
}

impl From<MemTransactionStorageItem> for TransactionTransfer {
    fn from(value: MemTransactionStorageItem) -> Self {
        TransactionTransfer {
            id: value.id,
            action: value.action,
            account_name: value.account_name,
        }
    }
}

impl AccountStorage for MemAccountStorage {
    fn create_account(&mut self, raw_data: AccountTransfer) -> Result<AccountTransfer, Error> {
        match self.storage.entry(raw_data.name.clone()) {
            std::collections::hash_map::Entry::Occupied(_) => Err(Error::AccountAlreadyExists),
            std::collections::hash_map::Entry::Vacant(vacant) => {
                let inserted = vacant.insert(raw_data);
                Ok((*inserted).clone())
            }
        }
    }

    fn get_account(&self, name: String) -> Result<AccountTransfer, Error> {
        match self.storage.get(&name) {
            Some(acc) => Ok(acc.clone()),
            None => Err(Error::AccountNotExists),
        }
    }

    fn update_account(&mut self, raw_data: AccountTransfer) -> Result<AccountTransfer, Error> {
        let key = raw_data.name.clone();
        match self.storage.entry(key.clone()) {
            std::collections::hash_map::Entry::Occupied(mut occ) => {
                occ.insert(raw_data);
            }
            std::collections::hash_map::Entry::Vacant(_) => return Err(Error::AccountNotExists),
        }

        Ok(self.storage.get(&key).unwrap().clone())
    }

    fn fee_account(&self) -> Result<AccountTransfer, Error> {
        match self.get_account(self.fee_acc_name.clone()) {
            Ok(acc) => Ok(acc),
            Err(err) => Err(err),
        }
    }

    fn accounts(&self) -> Result<Vec<AccountTransfer>, Error> {
        Ok(self.storage.values().cloned().collect())
    }
}

impl TransactionStorage for MemTransactionStorage {
    fn create_transaction(
        &mut self,
        account_name: String,
        action: TransactionAction,
    ) -> Result<TransactionTransfer, Error> {
        self.last_tr_id += 1;
        let item = MemTransactionStorageItem {
            id: self.last_tr_id,
            action,
            account_name,
        };
        self.storage.push(item.clone());
        Ok(TransactionTransfer::from(item))
    }

    fn transactions(&self) -> Result<Vec<TransactionTransfer>, Error> {
        Ok(self
            .storage
            .clone()
            .into_iter()
            .map(TransactionTransfer::from)
            .collect())
    }

    // O(n); n - number of transactions
    fn transaction_by_id(&self, id: usize) -> Result<TransactionTransfer, Error> {
        match self.storage.get(id - 1) {
            Some(item) => Ok(TransactionTransfer::from(item.clone())),
            None => Err(Error::AccountNotExists),
        }
    }
}

#[cfg(test)]
mod tests {

    use crate::bank::account::{Account, Error as AccError};
    use crate::bank::storage::Error as StorageError;
    use crate::bank::transactions::Transaction;

    use super::*;

    #[test]
    fn test_storage_get_account() {
        let mut storage = MemAccountStorage::new().unwrap();
        let test_name = "test".to_string();

        // test empty get
        assert_eq!(storage.get_account(test_name.clone()).is_err(), true);

        // test success insert
        let raw = AccountTransfer {
            name: test_name.clone(),
            balance: 0,
            trs: Default::default(),
        };
        assert_eq!(storage.create_account(raw).is_ok(), true);

        let result = storage.get_account(test_name.clone());
        assert_eq!(
            result.unwrap(),
            storage.storage.get(&test_name).unwrap().clone()
        );
    }

    #[test]
    fn test_storage_create_account() {
        let mut storage = MemAccountStorage::new().unwrap();
        let test_name = "test".to_string();

        // test add new acc (not existed early)
        let mut raw = AccountTransfer {
            name: test_name.clone(),
            balance: 0,
            trs: Default::default(),
        };
        assert_eq!(storage.create_account(raw).is_ok(), true);

        // test create acc with same name
        raw = AccountTransfer {
            name: test_name.clone(),
            balance: 0,
            trs: Default::default(),
        };
        let result = storage.create_account(raw);
        assert_eq!(result.is_err(), true);
        assert_eq!(result.err().unwrap(), StorageError::AccountAlreadyExists);
    }

    #[test]
    fn test_storage_update_account() {
        let mut storage = MemAccountStorage::new().unwrap();
        let test_name = "test".to_string();

        // updates non existed account
        let raw = AccountTransfer {
            name: "not_exist".to_string(),
            balance: 0,
            trs: Default::default(),
        };
        let result = storage.update_account(raw);
        assert_eq!(result.is_err(), true);
        assert_eq!(result.err().unwrap(), StorageError::AccountNotExists);

        // test add new acc (not existed early)
        let raw = AccountTransfer {
            name: test_name.clone(),
            balance: 0,
            trs: Default::default(),
        };
        let acc = storage.create_account(raw).unwrap();

        let to_update = AccountTransfer {
            name: acc.name.clone(),
            balance: 123,
            trs: Default::default(),
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
            .create_transaction(account_name.clone(), TransactionAction::Registration)
            .unwrap();
        assert_eq!(res.id, 1);
        assert_eq!(res.action, TransactionAction::Registration);
        assert_eq!(storage.storage.get(res.id - 1).unwrap().id, 1);

        res = storage
            .create_transaction(account_name.clone(), TransactionAction::Registration)
            .unwrap();
        assert_eq!(res.id, 2);
        assert_eq!(res.action, TransactionAction::Registration);
        assert_eq!(storage.storage.get(res.id - 1).unwrap().id, 2)
    }

    #[test]
    fn test_storage_transactions() {
        let mut storage = MemTransactionStorage::new();
        storage
            .create_transaction("test_1".to_string(), TransactionAction::Registration)
            .unwrap();
        storage
            .create_transaction("test_1".to_string(), TransactionAction::Registration)
            .unwrap();
        storage
            .create_transaction("test_2".to_string(), TransactionAction::Add(13))
            .unwrap();
        storage
            .create_transaction("test_3".to_string(), TransactionAction::Withdraw(13))
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
            .create_transaction("test_1".to_string(), TransactionAction::Registration)
            .unwrap();
        storage
            .create_transaction("test_2".to_string(), TransactionAction::Registration)
            .unwrap();
        storage
            .create_transaction("test_1".to_string(), TransactionAction::Add(13))
            .unwrap();
        storage
            .create_transaction("test_3".to_string(), TransactionAction::Withdraw(13))
            .unwrap();
        storage
            .create_transaction("test_3".to_string(), TransactionAction::Withdraw(11))
            .unwrap();

        assert_eq!(
            storage
                .transactions()
                .unwrap()
                .iter()
                .filter(|x| x.account_name == "test_1")
                .collect::<Vec<&TransactionTransfer>>()
                .len(),
            2
        );
        assert_eq!(
            storage
                .transactions()
                .unwrap()
                .iter()
                .filter(|x| x.account_name == "test_2")
                .collect::<Vec<&TransactionTransfer>>()
                .len(),
            1
        );
        assert_eq!(
            storage
                .transactions()
                .unwrap()
                .iter()
                .filter(|x| x.account_name == "test_3")
                .collect::<Vec<&TransactionTransfer>>()
                .len(),
            2
        );
    }

    #[test]
    fn test_storage_get_transaction_by_id() {
        let mut storage = MemTransactionStorage::new();
        storage
            .create_transaction("account_name".to_string(), TransactionAction::Registration)
            .unwrap();
        storage
            .create_transaction("account_name".to_string(), TransactionAction::Registration)
            .unwrap();
        storage
            .create_transaction("account_name".to_string(), TransactionAction::Add(15))
            .unwrap();

        assert_eq!(storage.transaction_by_id(1).unwrap().id, 1);
        assert_eq!(
            storage.transaction_by_id(1).unwrap().action,
            TransactionAction::Registration
        );

        assert_eq!(storage.transaction_by_id(2).unwrap().id, 2);
        assert_eq!(
            storage.transaction_by_id(2).unwrap().action,
            TransactionAction::Registration
        );

        assert_eq!(storage.transaction_by_id(3).unwrap().id, 3);
        assert_eq!(
            storage.transaction_by_id(3).unwrap().action,
            TransactionAction::Add(15)
        );

        assert_eq!(storage.transaction_by_id(4).is_err(), true);
    }

    #[test]
    fn test_account_new() {
        let mut acc_storage = MemAccountStorage::new().unwrap();
        let mut tr_storage = MemTransactionStorage::new();
        let target_name = "test".to_string();

        // test create account with new name
        let mut acc = Account::new(target_name.clone(), &mut acc_storage, &mut tr_storage);
        assert_eq!(acc.is_ok(), true);

        // test error to create acc with same name
        acc = Account::new(target_name.clone(), &mut acc_storage, &mut tr_storage);
        assert_eq!(acc.is_err(), true);

        // test transactions
        let trs = tr_storage
            .transactions()
            .unwrap()
            .into_iter()
            .filter(|x| x.account_name == target_name)
            .collect::<Vec<TransactionTransfer>>();

        assert_eq!(trs.len(), 1);
        assert_eq!(trs[0].action, TransactionAction::Registration)
    }

    #[test]
    fn test_account_inc_balance() {
        let mut acc_storage = MemAccountStorage::new().unwrap();
        let mut tr_storage = MemTransactionStorage::new();
        let target_name = "test".to_string();

        let mut acc = Account::new(target_name.clone(), &mut acc_storage, &mut tr_storage).unwrap();
        let _ = acc
            .inc_balance(10, &mut acc_storage, &mut tr_storage)
            .unwrap();
        assert_eq!(acc.balance(), 10);

        assert_eq!(
            acc.inc_balance(0, &mut acc_storage, &mut tr_storage)
                .err()
                .unwrap(),
            AccError::EmptyTransaction
        );
    }

    #[test]
    fn test_account_decr_balance() {
        let mut acc_storage = MemAccountStorage::new().unwrap();
        let mut tr_storage = MemTransactionStorage::new();
        let target_name = "test".to_string();
        let mut acc = Account::new(target_name.clone(), &mut acc_storage, &mut tr_storage).unwrap();
        acc.inc_balance(100, &mut acc_storage, &mut tr_storage)
            .unwrap();
        let tr = acc
            .decr_balance(10, &mut acc_storage, &mut tr_storage)
            .unwrap();
        assert_eq!(acc.balance(), 90);

        let trs = tr_storage
            .transactions()
            .unwrap()
            .into_iter()
            .filter(|x| x.account_name == "test")
            .collect::<Vec<TransactionTransfer>>();

        assert_eq!(trs.len(), 3);
        assert_eq!(trs[2].action, TransactionAction::Withdraw(10));
        assert_eq!(tr, Transaction::from(trs[2].clone()));
    }

    #[test]
    fn test_account_transaction() {
        let mut acc_storage = MemAccountStorage::new().unwrap();
        let mut tr_storage = MemTransactionStorage::new();
        let mut acc_f =
            Account::new("person_1".to_owned(), &mut acc_storage, &mut tr_storage).unwrap();
        let mut acc_s =
            Account::new("person_2".to_owned(), &mut acc_storage, &mut tr_storage).unwrap();

        let _ = acc_f
            .inc_balance(100, &mut acc_storage, &mut tr_storage)
            .unwrap();
        let tr = acc_f
            .make_transaction(10, &mut acc_s, None, &mut acc_storage, &mut tr_storage)
            .unwrap();
        assert_eq!(acc_f.balance(), 90);
        assert_eq!(acc_s.balance(), 10);

        let tr_t = tr_storage.transaction_by_id(tr.id).unwrap();
        assert_eq!(Transaction::from(tr_t.clone()), tr);
        assert_eq!(
            tr_t.action,
            TransactionAction::Transfer {
                to: "person_2".to_owned(),
                value: 10,
                fee: 0
            }
        );

        assert_eq!(acc_storage.fee_account().unwrap().balance, 0);

        // tr with fees
        let _ = acc_f
            .make_transaction(10, &mut acc_s, Some(10), &mut acc_storage, &mut tr_storage)
            .unwrap();
        assert_eq!(acc_f.balance(), 70);
        assert_eq!(acc_storage.fee_account().unwrap().balance, 10);
    }

    #[test]
    fn test_account_restore() {
        let mut acc_storage = MemAccountStorage::new().unwrap();
        let mut tr_storage = MemTransactionStorage::new();
        let acc_name = "person_1".to_owned();
        let mut acc_f = Account::new(acc_name.clone(), &mut acc_storage, &mut tr_storage).unwrap();
        let mut trs = Vec::new();
        trs.push(
            acc_f
                .inc_balance(10, &mut acc_storage, &mut tr_storage)
                .unwrap(),
        );
        trs.push(
            acc_f
                .decr_balance(5, &mut acc_storage, &mut tr_storage)
                .unwrap(),
        );
        trs.push(
            acc_f
                .inc_balance(1, &mut acc_storage, &mut tr_storage)
                .unwrap(),
        );
        trs.push(
            acc_f
                .inc_balance(20, &mut acc_storage, &mut tr_storage)
                .unwrap(),
        );

        let _ = acc_storage.update_account(AccountTransfer {
            name: "person_1".to_owned(),
            balance: 0,
            trs: Default::default(),
        });

        // test account exists
        let res = Account::from_transactions(acc_name.clone(), trs.clone(), &mut acc_storage);
        assert_eq!(res.unwrap().balance(), 26);

        // test transactions for account not existed
        let res =
            Account::from_transactions("not_exists".to_owned(), trs.clone(), &mut acc_storage);
        assert_eq!(res.is_ok(), true);
    }
}
