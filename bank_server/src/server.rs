use std::{sync::Arc, thread::JoinHandle};
use tokio::io::BufReader;
use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
    sync::Mutex,
};

use bank_protocol::types::{Request, RequestSerializer};

use serde_json::Value;

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
    // channel to send resp to handler
    pub resp_sender: tokio::sync::mpsc::Sender<String>,
}

// server
pub struct Server {
    host: String,
    port: usize,
    // channel to send msgs to handler
    handler_send: tokio::sync::mpsc::Sender<HandleItem>,
}

impl Server {
    pub fn new(
        host: Option<String>,
        port: Option<usize>,
        handler_send: tokio::sync::mpsc::Sender<HandleItem>,
    ) -> Self {
        Self {
            host: host.unwrap_or("127.0.0.1".to_string()),
            port: port.unwrap_or(8080),
            handler_send,
        }
    }

    // handle async connection
    pub async fn handle_connection(
        stream: TcpStream,
        send: tokio::sync::mpsc::Sender<HandleItem>,
        addr: String,
    ) {
        let mut reader = BufReader::new(stream);
        let (resp_sender, mut resp_reader) = tokio::sync::mpsc::channel::<String>(1);
        loop {
            let mut buf = String::new();
            tokio::select! {
                resp = resp_reader.recv() => {
                    reader.write_all(resp.unwrap().as_bytes()).await.unwrap();
                    reader.write_all(b"\n").await.unwrap();
                },
                _ = reader.read_line(&mut buf) => {
                    if buf.is_empty() {
                        println!("Client {addr} disconnected");
                        return
                    };
                    println!("Request received. Client {addr}. Msg: {buf}");
                    match serde_json::from_str::<RequestSerializer<Value>>(buf.as_str()) {
                        Ok(req) => {
                            let to_send = HandleItem {
                                req: Request::try_from(req).unwrap(),
                                resp_sender: resp_sender.clone(),
                            };
                            send.send(to_send).await.unwrap();
                            println!("Msg req sent to handler");
                        }
                        Err(_) => {
                            reader.write_all(b"wrong msg format\n").await.unwrap();
                            continue;
                        }
                    };
                }

            };
        }
    }

    // runs sync server
    pub async fn run(server: Arc<Mutex<Self>>) -> Result<JoinHandle<()>, Error> {
        let addr = {
            let q_s = server.lock().await;
            format!("{}:{}", q_s.host, q_s.port)
        };
        let listener = TcpListener::bind(addr.clone()).await.unwrap();
        println!("Bank one thread server started on: {addr}");
        loop {
            let (stream, addr) = listener.accept().await?;
            println!("New connection {addr}");
            let send = { server.lock().await.handler_send.clone() };
            tokio::spawn(async move {
                let _ = Self::handle_connection(stream, send, addr.to_string()).await;
            });
        }
    }
}
