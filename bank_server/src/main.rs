use std::sync::Arc;

use bank_core::bank::{
    implements::memory::storage::{MemAccountStorage, MemTransactionStorage},
    Bank,
};
use bank_server::{
    handler::Handler,
    server::{HandleItem, Server},
};
use tokio::sync::Mutex;

#[tokio::main]
async fn main() {
    // todo add args to host and port
    let (sender, recv) = tokio::sync::mpsc::channel::<HandleItem>(32);
    let acc_storage = MemAccountStorage::new().unwrap();
    let tr_storage = MemTransactionStorage::new();
    let bank = Bank::new(acc_storage, tr_storage, Some(3));
    let handler = Handler::new(bank, recv);
    let server = Server::new(Some("127.0.0.1".to_string()), Some(3000), sender);

    tokio::spawn(async move {
        let _ = Handler::run(Arc::new(Mutex::new(handler))).await;
    });
    let _ = tokio::join!(async move {
        let _ = Server::run(Arc::new(Mutex::new(server))).await;
    });
}
