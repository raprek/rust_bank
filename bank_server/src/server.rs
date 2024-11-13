use std::{
    io::{BufRead, BufReader, Write},
    net::{SocketAddr, TcpListener, TcpStream},
    sync::{Arc, Mutex, RwLock},
    thread::{self, JoinHandle},
    time::Duration,
};

use bank_core::bank::storage::{AccountStorage, TransactionStorage};
use bank_protocol::types::{Request, RequestSerializer};
use chan::{Receiver, Sender};
use serde::ser;
use serde_json::{to_string, Value};

use crate::handler::{self, Handler};

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

#[derive(Debug)]
pub struct HandleItem {
    pub req: Request<Value>,
    pub stream: TcpStream,
}

// server
pub struct Server {
    host: String,
    port: usize,
    timeout: Option<Duration>,
    handler_send: Sender<HandleItem>,
}

impl Server {
    pub fn new(
        host: Option<String>,
        port: Option<usize>,
        timeout: Option<Duration>,
        handler_send: Sender<HandleItem>,
    ) -> Self {
        Self {
            host: host.unwrap_or("127.0.0.1".to_string()),
            port: port.unwrap_or(8080),
            timeout,
            handler_send,
        }
    }

    pub fn handle_connection(
        mut stream: TcpStream,
        send: Sender<HandleItem>,
        addr: String,
    ) -> Result<(), std::io::Error> {
        loop {
            let mut req = String::new();
            let mut reader = BufReader::new(stream.try_clone().unwrap());
            reader.read_line(&mut req)?;
            if req.len() == 0 {
                println!("Client {addr} disconnected");
                return Ok(());
            }

            // unpack request
            println!("Request received. Client {addr}. Msg: {req}");
            match serde_json::from_str::<RequestSerializer<Value>>(req.as_str()) {
                Ok(req) => {
                    let to_send = HandleItem {
                        req: Request::try_from(req).unwrap(),
                        stream: stream.try_clone().unwrap(),
                    };
                    send.send(to_send);

                    println!("Msg req sent to handler");
                }
                Err(_) => {
                    stream.write_all(b"wrong msg format\n")?;
                    continue;
                }
            };
        }
    }

    // runs sync server
    pub fn run(server: Arc<Mutex<Self>>) -> Result<JoinHandle<()>, Error> {
        Ok(thread::spawn(move || {
            let addr = {
                let q_s = server.lock().unwrap();
                format!("{}:{}", q_s.host, q_s.port)
            };
            let listener = TcpListener::bind(&addr).unwrap();
            println!("Bank one thread server started on: {addr}");
            loop {
                if let Ok((mut stream, addr)) = listener.accept() {
                    println!("New client. Client {addr}");
                    let timeout = { server.lock().unwrap().timeout };
                    stream.set_read_timeout(timeout).unwrap();
                    stream.set_write_timeout(timeout).unwrap();
                    let addr = addr.to_string();
                    let send = { server.lock().unwrap().handler_send.clone() };
                    // read response
                    thread::spawn(|| {
                        let _ = Self::handle_connection(stream, send, addr);
                    });
                } else {
                    print!("Error getting connection")
                }
            }
        }))
    }
}
