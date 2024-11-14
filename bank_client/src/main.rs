use std::time::Duration;

use bank_client::client::Client;

#[tokio::main]
async fn main() {
    let client = Client::new("127.0.0.1:3000".to_string(), Duration::from_secs(30));

    // create account
    // first shows suc creating of acc
    // sec shows error "account already exists"
    println!("Create an account");
    for _ in 0..2 {
        match client.create_account("test_acc".to_string()).await {
            Ok(acc) => println!("Acc created: {:?}", acc),
            Err(err) => println!("Error creating account, error: {:?}", err),
        }
    }

    // increments acc balance
    println!("Increment  balance");
    match client.incr_balance("test_acc".to_string(), 50).await {
        Ok(tr) => println!("Balance incremented, tr_id: {:?}", tr),
        Err(err) => println!("Error incrementing account balance, error: {:?}", err),
    }

    // decremets acc balance
    println!("Decrement  balance");
    match client.decr_balance("test_acc".to_string(), 20).await {
        Ok(tr) => println!("Balance decremented, tr_id: {:?}", tr),
        Err(err) => println!("Error decremented account balance, error: {:?}", err),
    }

    // make transaction
    println!("Make transaction");
    match client.create_account("test_acc_2".to_string()).await {
        Ok(tr) => println!("Acc created: {:?}", tr),
        Err(err) => println!("Error creating account, error: {:?}", err),
    }

    match client.make_transaction("test_acc".to_string(), "test_acc_2".to_string(), 10).await {
        Ok(tr) => println!("Transaction made, tr_id: {:?}", tr),
        Err(err) => println!("Error making transaction, error: {:?}", err),
    }

    // get transaction
    println!("Get transaction. id: 1");
    match client.transaction(1).await {
        Ok(trs) => println!("Transaction {:?}", trs),
        Err(err) => println!("Error getting transaction, error: {:?}", err),
    }

    // show transactions
    println!("Get transactions");
    match client.transactions().await {
        Ok(trs) => println!("Transactions {:?}", trs),
        Err(err) => println!("Error getting transaction, error: {:?}", err),
    }

    println!("Get account transactions. Acc name: test_acc");
    match client.account_transactions("test_acc".to_owned()).await {
        Ok(trs) => println!("Transactions {:?}", trs),
        Err(err) => println!("Error getting transaction, error: {:?}", err),
    }

    println!("Get account balance. Acc name: test_acc");
    match client.account_balance("test_acc".to_owned()).await {
        Ok(trs) => println!("Account balance {:?}", trs),
        Err(err) => println!("Error getting acc balance, error: {:?}", err),
    }
}
