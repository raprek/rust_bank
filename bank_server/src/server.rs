use std::{
    io::{BufRead, BufReader, Write},
    net::TcpListener,
    time::Duration,
};

use bank_core::bank::storage::{AccountStorage, TransactionStorage};
use bank_protocol::types::{Request, RequestSerializer};
use serde_json::Value;

use crate::handler::Handler;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("io error: `{0}`")]
    IOError(String),
}

impl From<std::io::Error> for Error {
    fn from(value: std::io::Error) -> Self {
        Self::IOError(value.to_string())
    }
}

// server
pub struct Server<A: AccountStorage + Default, T: TransactionStorage + Default> {
    handler: Handler<A, T>,
    host: String,
    port: usize,
    timeout: Option<Duration>,
}

impl<A: AccountStorage + Default, T: TransactionStorage + Default> Server<A, T> {
    pub fn new(
        handler: Handler<A, T>,
        host: Option<String>,
        port: Option<usize>,
        timeout: Option<Duration>,
    ) -> Self {
        Self {
            handler,
            host: host.unwrap_or("127.0.0.1".to_string()),
            port: port.unwrap_or(8080),
            timeout,
        }
    }

    // runs sync server
    pub fn run(&mut self) -> Result<(), Error> {
        let addr = format!("{}:{}", self.host, self.port);
        let listener = TcpListener::bind(&addr).unwrap();
        println!("Bank one thread server started on: {addr}");
        loop {
            if let Ok((mut stream, addr)) = listener.accept() {
                println!("New client. Client {addr}");
                stream.set_read_timeout(self.timeout).unwrap();
                stream.set_write_timeout(self.timeout).unwrap();

                // read response
                let mut response = String::new();
                let mut reader = BufReader::new(&stream);
                reader.read_line(&mut response)?;

                // unpack request
                println!("Request received. Client {addr}. Msg: {response}");
                let msg: RequestSerializer<Value> = match serde_json::from_str(response.as_str()) {
                    Ok(msg) => msg,
                    Err(_) => {
                        let _ = stream.write_all(b"wrong msg format");
                        continue;
                    }
                };
                let req = Request::try_from(msg).unwrap();
                match self.handler.handle_msg(req, stream) {
                    Ok(_) => println!("Req SUCCESSFULLY processed. Client {addr}"),
                    Err(err) => println!("Req FAILED processed. Client {addr}. Error: {err}"),
                }
            } else {
                print!("Error getting connection")
            }
        }
    }
}
