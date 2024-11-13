use std::sync::{Arc, Mutex};

use bank_core::bank::{
    implements::memory::storage::{MemAccountStorage, MemTransactionStorage},
    Bank,
};
use bank_server::{
    handler::Handler,
    server::{HandleItem, Server},
};

fn main() {
    // todo add args to host and port
    let acc_storage = MemAccountStorage::new().unwrap();
    let tr_storage = MemTransactionStorage::new();
    let (sender, recv) = chan::sync::<HandleItem>(1);
    let bank = Bank::new(acc_storage, tr_storage, Some(3));
    let handler = Handler::new(bank, recv);
    let server = Server::new(Some("127.0.0.1".to_string()), Some(3000), None, sender);

    let _ = Handler::run(Arc::new(Mutex::new(handler)));
    let s_t = Server::run(Arc::new(Mutex::new(server))).unwrap();
    s_t.join().unwrap();
}
