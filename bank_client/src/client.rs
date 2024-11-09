use std::fmt::Display;
use std::io::{BufRead, BufReader, Read};
use std::time::Duration;
use std::{io::Write, net::TcpStream, vec::Vec};

use bank_protocol::types::{
    Method, Request, RequestAccountTransactionsPayload, RequestCreateAccountPayload,
    RequestDecrBalancePayload, RequestIncrBalancePayload, RequestMakeTransactionPayload,
    RequestSerializer, RequestTransactionByIdPayload, RequestTransactionsPayload, Response,
    ResponseAccountPayload, ResponseErrorPayload, ResponseSerializer, ResponseShortTrPayload,
    ResponseTrPayload, ResponseTrsPayload, TransactionSerializer,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;

pub struct Client {
    server_addr: String,
    timeout: Duration,
}

#[derive(Debug)]
pub struct Account {
    pub name: String,
    pub balance: usize,
}

#[derive(Debug)]
pub enum TransactionAction {
    Registration,
    Add(usize),
    Withdraw(usize),
    Transfer {
        to: String, // account id
        value: usize,
        fee: usize,
    },
}

#[derive(Debug)]
pub struct Transaction {
    pub id: usize,
    pub action: TransactionAction,
    pub account_name: String,
}

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("connection error: `{0}`")]
    ConnectionError(String),
    #[error("invalid msg format: `{0}`")]
    InvalidMsg(String),
    #[error("server error: `{0}`")]
    ServerError(String),
}

impl From<std::io::Error> for Error {
    fn from(value: std::io::Error) -> Self {
        Self::ConnectionError(value.to_string())
    }
}

impl From<serde_json::Error> for Error {
    fn from(value: serde_json::Error) -> Self {
        Self::InvalidMsg(value.to_string())
    }
}

impl From<uuid::Error> for Error {
    fn from(value: uuid::Error) -> Self {
        Self::InvalidMsg(value.to_string())
    }
}

impl From<ResponseAccountPayload> for Account {
    fn from(value: ResponseAccountPayload) -> Self {
        Self {
            name: value.name,
            balance: value.balance,
        }
    }
}

impl From<TransactionSerializer> for Transaction {
    fn from(value: TransactionSerializer) -> Self {
        let action = match value.action {
            bank_protocol::types::TransactionActionSerializer::Registration => {
                TransactionAction::Registration
            }
            bank_protocol::types::TransactionActionSerializer::Add(value) => {
                TransactionAction::Add(value)
            }
            bank_protocol::types::TransactionActionSerializer::Withdraw(value) => {
                TransactionAction::Withdraw(value)
            }
            bank_protocol::types::TransactionActionSerializer::Transfer { to, value, fee } => {
                TransactionAction::Transfer { to, value, fee }
            }
        };
        Self {
            id: value.id,
            action,
            account_name: value.account_name,
        }
    }
}

// impl From<Transaction>

impl Client {
    pub fn new(server_addr: String, timeout: Duration) -> Self {
        Self {
            server_addr,
            timeout,
        }
    }

    pub fn send_request<R: Serialize>(&self, req: Request<R>) -> Result<Response<Value>, Error> {
        // set timeout
        let mut stream = TcpStream::connect(self.server_addr.clone())?;
        stream.set_read_timeout(Some(self.timeout)).unwrap();
        stream.set_write_timeout(Some(self.timeout)).unwrap();

        // write resp
        serde_json::to_writer(&stream, &RequestSerializer::from(req))?;
        stream.write(b"\n")?;
        stream.flush()?;

        // wait resp
        let buf_reader = BufReader::new(&mut stream);
        let res: String = buf_reader
            .lines()
            .map(|result| result.unwrap())
            .take_while(|line| !line.is_empty())
            .collect();

        Ok(Response::try_from(serde_json::from_str::<
            ResponseSerializer<Value>,
        >(&res.as_str())?)?)
    }

    pub fn create_account(&self, account_name: String) -> Result<Account, Error> {
        let req = Request::new(
            Method::CreteAccount,
            RequestCreateAccountPayload { account_name },
        );
        let resp = self.send_request(req)?;
        match resp.code {
            bank_protocol::types::RespCode::OK => {
                let payload: ResponseAccountPayload =
                    serde_json::from_value(resp.payload.unwrap())?;
                Ok(Account::from(payload))
            }
            bank_protocol::types::RespCode::ERR => {
                let payload: ResponseErrorPayload = serde_json::from_value(resp.payload.unwrap())?;
                Err(Error::ServerError(payload.error))
            }
        }
    }

    // increments acc balance. Returns transaction id
    pub fn incr_balance(&self, account_name: String, value: usize) -> Result<usize, Error> {
        let req = Request::new(
            Method::IncrBalance,
            RequestIncrBalancePayload {
                account_name,
                value,
            },
        );
        let resp = self.send_request(req)?;
        match resp.code {
            bank_protocol::types::RespCode::OK => {
                let payload: ResponseShortTrPayload =
                    serde_json::from_value(resp.payload.unwrap())?;
                Ok(payload.id)
            }
            bank_protocol::types::RespCode::ERR => {
                let payload: ResponseErrorPayload = serde_json::from_value(resp.payload.unwrap())?;
                Err(Error::ServerError(payload.error))
            }
        }
    }

    // decrements acc balance. Returns transaction id
    pub fn decr_balance(&self, account_name: String, value: usize) -> Result<usize, Error> {
        let req = Request::new(
            Method::DecrBalance,
            RequestDecrBalancePayload {
                account_name,
                value,
            },
        );
        let resp = self.send_request(req)?;
        match resp.code {
            bank_protocol::types::RespCode::OK => {
                let payload: ResponseShortTrPayload =
                    serde_json::from_value(resp.payload.unwrap())?;
                Ok(payload.id)
            }
            bank_protocol::types::RespCode::ERR => {
                let payload: ResponseErrorPayload = serde_json::from_value(resp.payload.unwrap())?;
                Err(Error::ServerError(payload.error))
            }
        }
    }

    // decrements acc balance. Returns transaction id
    pub fn make_transaction(
        &self,
        account_name: String,
        account_to_name: String,
        value: usize,
    ) -> Result<usize, Error> {
        let req = Request::new(
            Method::MakeTransaction,
            RequestMakeTransactionPayload {
                account_name,
                value,
                account_to_name,
            },
        );
        let resp = self.send_request(req)?;
        match resp.code {
            bank_protocol::types::RespCode::OK => {
                let payload: ResponseShortTrPayload =
                    serde_json::from_value(resp.payload.unwrap())?;
                Ok(payload.id)
            }
            bank_protocol::types::RespCode::ERR => {
                let payload: ResponseErrorPayload = serde_json::from_value(resp.payload.unwrap())?;
                Err(Error::ServerError(payload.error))
            }
        }
    }

    pub fn transaction(&self, id: usize) -> Result<Transaction, Error> {
        let req = Request::new(Method::Transaction, RequestTransactionByIdPayload { id });
        let resp = self.send_request(req)?;
        match resp.code {
            bank_protocol::types::RespCode::OK => {
                let payload: ResponseTrPayload = serde_json::from_value(resp.payload.unwrap())?;
                Ok(Transaction::from(payload.tr))
            }
            bank_protocol::types::RespCode::ERR => {
                let payload: ResponseErrorPayload = serde_json::from_value(resp.payload.unwrap())?;
                Err(Error::ServerError(payload.error))
            }
        }
    }

    pub fn transactions(&self) -> Result<Vec<Transaction>, Error> {
        let req = Request::new(Method::Transactions, RequestTransactionsPayload {});
        let resp = self.send_request(req)?;
        match resp.code {
            bank_protocol::types::RespCode::OK => {
                let payload: ResponseTrsPayload = serde_json::from_value(resp.payload.unwrap())?;
                Ok(payload.trs.into_iter().map(Transaction::from).collect())
            }
            bank_protocol::types::RespCode::ERR => {
                let payload: ResponseErrorPayload = serde_json::from_value(resp.payload.unwrap())?;
                Err(Error::ServerError(payload.error))
            }
        }
    }

    pub fn account_transactions(&self, account_name: String) -> Result<Vec<Transaction>, Error> {
        let req = Request::new(
            Method::AccountTransactions,
            RequestAccountTransactionsPayload { account_name },
        );
        let resp = self.send_request(req)?;
        match resp.code {
            bank_protocol::types::RespCode::OK => {
                let payload: ResponseTrsPayload = serde_json::from_value(resp.payload.unwrap())?;
                Ok(payload.trs.into_iter().map(Transaction::from).collect())
            }
            bank_protocol::types::RespCode::ERR => {
                let payload: ResponseErrorPayload = serde_json::from_value(resp.payload.unwrap())?;
                Err(Error::ServerError(payload.error))
            }
        }
    }
}
