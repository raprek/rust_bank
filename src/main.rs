use rust_bank::bank::{
    implements::memory::storage::{MemAccountStorage, MemTransactionStorage},
    Bank,
};

fn main() {
    let tr_fee = 1;

    // init base storage
    let mut bank = Bank::new(
        MemAccountStorage::new().unwrap(),
        MemTransactionStorage::new(),
        Some(tr_fee),
    );

    // create acc
    let mut acc = bank.create_account("some_acc".to_string()).unwrap();
    println!("Created an account: {acc}");

    // incr balance
    let _ = bank.inc_acc_balance(&mut acc, 10);
    println!("Account after increment on 10: {acc}");

    // decr balance
    let _ = bank.decr_acc_balance(&mut acc, 2);
    println!("Account after decrement balance on 2: {acc}");

    // transaction
    let mut to_acc = bank.create_account("to_acc".to_string()).unwrap();

    let tr_amount = 3;
    println!(
        "Before transaction. Fee: {tr_fee}. Amount: {tr_amount} Account from: {acc}, to {to_acc}"
    );
    let _ = bank.make_transaction(&mut acc, &mut to_acc, tr_amount);
    println!(
        "After transaction. Fee: {tr_fee}. Amount: {tr_amount} Account from: {acc}, to {to_acc}"
    );

    println!("----------------------------");
    // transactions
    let trs = bank.account_transactions(acc.name.clone()).unwrap();
    println!("Show transactions for an account: {acc}");
    trs.iter().for_each(|tr| println!("Tr: {tr}"));
    println!("----------------------------");

    let trs = bank.account_transactions(to_acc.name.clone()).unwrap();
    println!("Show transactions for an account: {to_acc}");
    trs.iter().for_each(|tr| println!("Tr: {tr}"));
    println!("----------------------------");

    // trs restore

    let mut bank_sec = Bank::new(
        MemAccountStorage::new().unwrap(),
        MemTransactionStorage::new(),
        Some(tr_fee),
    );

    println!("Show accs in first bank:");
    bank.accounts()
        .unwrap()
        .into_iter()
        .for_each(|acc| println!("Acc: {acc}"));
    println!("----------------------------");

    println!("Show accs in sec bank before restore:");
    bank_sec
        .accounts()
        .unwrap()
        .into_iter()
        .for_each(|acc| println!("Acc: {acc}"));
    println!("----------------------------");

    let _ = bank_sec.restore_accounts_from_bank_transactions(&bank);
    println!("Show accs in sec bank after restore:");
    bank_sec
        .accounts()
        .unwrap()
        .into_iter()
        .for_each(|acc| println!("Acc: {acc}"));
    println!("----------------------------");
}
