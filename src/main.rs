use std::rc::Rc;

use rust_bank::bank::{
    base::{
        account::Account,
        storage::{Storage, TransactionStorage},
    },
    implements::memory::storage::{MemAccountStorage, MemTransactionStorage},
};

fn main() {
    // init base storage
    let storage = Rc::new(Storage::new(
        MemAccountStorage::new(),
        MemTransactionStorage::new(),
    ));

    // create acc
    let mut acc = Account::new("some_name".to_owned(), storage.clone()).unwrap();
    println!("Created an account: {acc}");

    // incr balance
    let _ = acc.inc_balance(10);
    println!("Account after increment on 10: {acc}");

    // decr balance
    let _ = acc.decr_balance(2);
    println!("Account after decrement balance on 2: {acc}");

    // transaction
    let mut to_acc = Account::new("some_name_2".to_owned(), storage.clone()).unwrap();
    let tr_fee = 1;
    let tr_amount = 3;
    println!(
        "Before transaction. Fee: {tr_fee}. Amount: {tr_amount} Account from: {acc}, to {to_acc}"
    );
    let _ = acc.make_transaction(tr_amount, &mut to_acc, Some(tr_fee));
    println!(
        "After transaction. Fee: {tr_fee}. Amount: {tr_amount} Account from: {acc}, to {to_acc}"
    );

    // transactions
    let trs = storage
        .clone()
        .tr_storage
        .borrow()
        .account_transactions(acc.name.clone())
        .unwrap();
    println!("Show transactions for an account: {acc}");
    trs.iter().for_each(|tr| println!("Tr: {tr}"));
}
