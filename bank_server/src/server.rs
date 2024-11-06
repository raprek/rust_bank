use std::{io::{Read, Write}, net::{TcpListener, TcpStream}};

use bank_core::bank::storage::{AccountStorage, TransactionStorage};
use bank_protocol::types::{Request, RequestSerializer};
use serde_json::Value;

use crate::handler::Handler;

pub enum Error {
    AcceptConnError,
    SerializeMsgError,
    HandleError,
}

pub struct Server<A: AccountStorage + Default, T: TransactionStorage + Default> {
    handler: Handler<A, T>,
    host: String,
    port: usize

}

impl <A: AccountStorage + Default, T: TransactionStorage + Default> Server<A, T> {
    pub fn new(handler: Handler<A, T>, host: Option<String>, port: Option<usize> ) -> Self {
        Self {
            handler: handler,
            host: host.unwrap_or("127.0.0.1".to_string()),
            port: port.unwrap_or(8080)
        }
    }

    pub fn run(&mut self) -> Result<(), Error> {
        let addr = format!("{}:{}", self.host, self.port);
        let listener = TcpListener::bind(&addr).unwrap();
        println!("Bank one thread server started on: {addr}");
        loop {
            if let Ok((mut stream, addr)) = listener.accept() {
                println!("New client. Client {addr}");
                let msg: RequestSerializer<Value> = match serde_json::from_reader(&stream) {
                    Ok(msg) => msg,
                    Err(_) => {
                        let _ = stream.write_all(b"wrong msg format");
                        continue;
                    },
                };
                let req = Request::try_from(msg).unwrap();
                self.handler.handle_msg(req, stream);
                println!("Req processed. Client {addr}");
            } 
            
        }
    }
}