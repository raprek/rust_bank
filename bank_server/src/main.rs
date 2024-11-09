use bank_core::bank::{
    implements::memory::storage::{MemAccountStorage, MemTransactionStorage},
    storage::AccountStorage,
    Bank,
};
use bank_server::{handler::Handler, server::Server};

fn main() {
    // todo add args to host and port
    let acc_storage = MemAccountStorage::new().unwrap();
    let tr_storage = MemTransactionStorage::new();
    let handler = Handler::new(Bank::new(acc_storage, tr_storage, Some(3)));
    let mut server = Server::new(handler, None, None, None);
    let _ = server.run();
}
