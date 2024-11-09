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

    use crate::bank::storage::Error as StorageError;
    use crate::bank::{Bank, Error as BankError, Transaction};

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
    fn test_bank_create_acc() {
        let acc_storage = MemAccountStorage::new().unwrap();
        let tr_storage = MemTransactionStorage::new();
        let mut bank = Bank::new(acc_storage, tr_storage, Some(0));
        let target_name = "test".to_string();

        // test create account with new name
        let mut acc = bank.create_account(target_name.clone());
        assert_eq!(acc.is_ok(), true);

        // test error to create acc with same name
        acc = bank.create_account(target_name.clone());
        assert_eq!(acc.is_err(), true);

        // test transactions
        let trs = bank
            .transactions()
            .unwrap()
            .into_iter()
            .filter(|x| x.account_name == target_name)
            .collect::<Vec<Transaction>>();

        assert_eq!(trs.len(), 1);
        assert_eq!(trs[0].action, TransactionAction::Registration)
    }

    #[test]
    fn test_bank_account_inc_balance() {
        let acc_storage = MemAccountStorage::new().unwrap();
        let tr_storage = MemTransactionStorage::new();
        let mut bank = Bank::new(acc_storage, tr_storage, Some(0));
        let target_name = "test".to_string();

        let _ = bank.create_account(target_name.clone());
        let _ = bank.inc_acc_balance(target_name.clone(), 10).unwrap();
        let acc = bank.account(target_name.clone()).unwrap();
        assert_eq!(acc.balance, 10);

        assert_eq!(
            bank.inc_acc_balance(target_name.clone(), 0).err().unwrap(),
            BankError::EmptyTransaction
        );
    }

    #[test]
    fn test_bank_account_decr_balance() {
        let acc_storage = MemAccountStorage::new().unwrap();
        let tr_storage = MemTransactionStorage::new();
        let mut bank = Bank::new(acc_storage, tr_storage, Some(0));
        let target_name = "test".to_string();

        let _ = bank.create_account(target_name.clone()).unwrap();
        bank.inc_acc_balance(target_name.clone(), 100).unwrap();
        let _ = bank.decr_acc_balance(target_name.clone(), 10).unwrap();

        let acc = bank.account(target_name.clone()).unwrap();
        assert_eq!(acc.balance, 90);

        let trs = bank
            .transactions()
            .unwrap()
            .into_iter()
            .filter(|x| x.account_name == "test")
            .collect::<Vec<Transaction>>();

        assert_eq!(trs.len(), 3);
        assert_eq!(trs[2].action, TransactionAction::Withdraw(10));
    }

    #[test]
    fn test_account_transaction() {
        let acc_storage = MemAccountStorage::new().unwrap();
        let tr_storage = MemTransactionStorage::new();
        let mut bank = Bank::new(acc_storage, tr_storage, Some(0));

        let _acc_f = bank.create_account("person_1".to_owned()).unwrap();
        let _acc_s = bank.create_account("person_2".to_owned()).unwrap();

        let _ = bank.inc_acc_balance("person_1".to_owned(), 100).unwrap();
        let tr_id = bank
            .make_transaction("person_1".to_owned(), "person_2".to_owned(), 10)
            .unwrap();
        let acc_f = bank.account("person_1".to_owned()).unwrap();
        let acc_s = bank.account("person_2".to_owned()).unwrap();
        assert_eq!(acc_f.balance, 90);
        assert_eq!(acc_s.balance, 10);

        let tr_t = bank.transaction_by_id(tr_id).unwrap();
        assert_eq!(
            tr_t.action,
            TransactionAction::Transfer {
                to: "person_2".to_owned(),
                value: 10,
                fee: 0
            }
        );

        assert_eq!(bank.acc_storage.fee_account().unwrap().balance, 0);

        let _ = bank
            .make_transaction("person_1".to_owned(), "person_2".to_owned(), 10)
            .unwrap();
        let acc_f = bank.account("person_1".to_owned()).unwrap();

        // tr with fees
        assert_eq!(acc_f.balance, 80);
    }

    #[test]
    fn test_account_restore() {
        let acc_storage = MemAccountStorage::new().unwrap();
        let tr_storage = MemTransactionStorage::new();
        let mut bank = Bank::new(acc_storage, tr_storage, Some(0));
        let account_name = "person_1".to_owned();
        let mut trs = Vec::new();
        trs.push(Transaction {
            id: 1,
            action: TransactionAction::Registration,
            account_name: account_name.clone(),
        });
        trs.push(Transaction {
            id: 2,
            action: TransactionAction::Add(10),
            account_name: account_name.clone(),
        });
        trs.push(Transaction {
            id: 3,
            action: TransactionAction::Withdraw(5),
            account_name: account_name.clone(),
        });
        trs.push(Transaction {
            id: 4,
            action: TransactionAction::Add(1),
            account_name: account_name.clone(),
        });
        trs.push(Transaction {
            id: 4,
            action: TransactionAction::Add(20),
            account_name: account_name.clone(),
        });

        // test account exists
        let res = bank
            .restore_account_from_transactions(account_name.clone(), trs)
            .unwrap();
        assert_eq!(res.balance, 26);
    }
}
