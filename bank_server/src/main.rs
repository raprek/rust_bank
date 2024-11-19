use bank_core::bank::{
    implements::memory::storage::{MemAccountStorage, MemTransactionStorage},
    Bank,
};
use bank_server::{
    handler::Handler,
    server::{HandleItem, Server},
};

#[tokio::main]
async fn main() {
    // todo add args to host and port
    let (sender, recv) = tokio::sync::mpsc::channel::<HandleItem>(32);
    let acc_storage = MemAccountStorage::new().unwrap();
    let tr_storage = MemTransactionStorage::new();
    let bank = Bank::new(acc_storage, tr_storage, Some(3));
    let handler = Handler::new(bank, recv);
    let server = Server::new(Some("127.0.0.1".to_string()), Some(3000), sender);
    let h_t = Handler::run(handler);
    let s_t = Server::run(server);
    tokio::select! {
        _ = s_t => println!("Server stopped"),
        _ = h_t => println!("Handler stopped"),
    };
}
