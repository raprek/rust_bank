use std::sync::{Arc, Mutex, RwLock};
use std::thread::{self, JoinHandle};
use std::{io::Write, net::TcpStream};

use bank_core::bank::{
    storage::{AccountStorage, TransactionAction, TransactionStorage},
    Account, Bank, Transaction,
};
use bank_protocol::types::{
    Request, RequestAccountTransactionsPayload, RequestBalancePayload, RequestCreateAccountPayload,
    RequestDecrBalancePayload, RequestIncrBalancePayload, RequestMakeTransactionPayload,
    RequestTransactionByIdPayload, Response, ResponseAccountPayload, ResponseBalancePayload,
    ResponseErrorPayload, ResponseSerializer, ResponseShortTrPayload, ResponseTrPayload,
    ResponseTrsPayload, TransactionActionSerializer, TransactionSerializer,
};
use chan::Receiver;
use serde_json::Value;

use crate::server::HandleItem;

#[derive(Debug)]
pub struct Handler<A: AccountStorage + Default, T: TransactionStorage + Default> {
    bank: Arc<RwLock<Bank<A, T>>>,
    recv_chan: Receiver<HandleItem>,
}

struct Tr(Transaction);

impl From<Tr> for TransactionSerializer {
    fn from(value: Tr) -> Self {
        let action = match value.0.action {
            TransactionAction::Registration => TransactionActionSerializer::Registration,
            TransactionAction::Add(value) => TransactionActionSerializer::Add(value),
            TransactionAction::Withdraw(value) => TransactionActionSerializer::Withdraw(value),
            TransactionAction::Transfer { to, value, fee } => {
                TransactionActionSerializer::Transfer { to, value, fee }
            }
        };
        Self {
            id: value.0.id,
            action,
            account_name: value.0.account_name,
        }
    }
}

impl<
        A: AccountStorage + Default + Send + Sync + 'static,
        T: TransactionStorage + Default + Send + Sync + 'static,
    > Handler<A, T>
{
    pub fn new(bank: Bank<A, T>, recv_chan: Receiver<HandleItem>) -> Self {
        Self {
            bank: Arc::new(RwLock::new(bank)),
            recv_chan,
        }
    }

    // runs server
    pub fn run(handler: Arc<Mutex<Self>>) -> JoinHandle<()> {
        println!("Handler started");
        thread::spawn(move || loop {
            let h_item = { handler.clone().lock().unwrap().recv_chan.recv().unwrap() };

            println!("New msg in handler {:?}", h_item.req);
            let bank = handler.clone().lock().unwrap().bank.clone();

            thread::spawn(move || {
                let _ = Self::handle_msg(bank, h_item.req, h_item.stream);
            });
        })
    }

    pub fn handle_msg(
        bank: Arc<RwLock<Bank<A, T>>>,
        req: Request<Value>,
        mut stream: TcpStream,
    ) -> Result<(), std::io::Error> {
        let req_id = req.id;
        let _ = match req.method {
            bank_protocol::types::Method::CreteAccount => {
                match Self::handle_create_account(bank, req) {
                    Ok(acc) => {
                        let payload = acc;
                        serde_json::to_writer(
                            &stream,
                            &ResponseSerializer::from(Response::<ResponseAccountPayload>::ok(
                                req_id,
                                Some(payload),
                            )),
                        )
                    }
                    Err(err) => serde_json::to_writer(
                        &stream,
                        &ResponseSerializer::from(err.to_response(req_id)),
                    ),
                }
            }
            bank_protocol::types::Method::IncrBalance => match Self::handle_incr_balance(bank, req)
            {
                Ok(id) => serde_json::to_writer(
                    &stream,
                    &ResponseSerializer::from(Response::ok(
                        req_id,
                        Some(ResponseShortTrPayload { id }),
                    )),
                ),
                Err(err) => serde_json::to_writer(
                    &stream,
                    &ResponseSerializer::from(err.to_response(req_id)),
                ),
            },
            bank_protocol::types::Method::DecrBalance => match Self::handle_decr_balance(bank, req)
            {
                Ok(id) => serde_json::to_writer(
                    &stream,
                    &ResponseSerializer::from(Response::ok(
                        req_id,
                        Some(ResponseShortTrPayload { id }),
                    )),
                ),
                Err(err) => serde_json::to_writer(
                    &stream,
                    &ResponseSerializer::from(err.to_response(req_id)),
                ),
            },
            bank_protocol::types::Method::MakeTransaction => {
                match Self::handler_make_transaction(bank, req) {
                    Ok(id) => serde_json::to_writer(
                        &stream,
                        &ResponseSerializer::from(Response::ok(
                            req_id,
                            Some(ResponseShortTrPayload { id }),
                        )),
                    ),
                    Err(err) => serde_json::to_writer(
                        &stream,
                        &ResponseSerializer::from(err.to_response(req_id)),
                    ),
                }
            }
            bank_protocol::types::Method::Transactions => match Self::handler_transactions(bank) {
                Ok(trs) => serde_json::to_writer(
                    &stream,
                    &ResponseSerializer::from(Response::ok(
                        req_id,
                        Some(ResponseTrsPayload { trs }),
                    )),
                ),
                Err(err) => serde_json::to_writer(
                    &stream,
                    &ResponseSerializer::from(err.to_response(req_id)),
                ),
            },
            bank_protocol::types::Method::Transaction => match Self::handler_transaction(bank, req)
            {
                Ok(tr) => serde_json::to_writer(
                    &stream,
                    &ResponseSerializer::from(Response::ok(req_id, Some(ResponseTrPayload { tr }))),
                ),
                Err(err) => serde_json::to_writer(
                    &stream,
                    &ResponseSerializer::from(err.to_response(req_id)),
                ),
            },
            bank_protocol::types::Method::AccountTransactions => {
                match Self::handler_account_trs(bank, req) {
                    Ok(trs) => serde_json::to_writer(
                        &stream,
                        &ResponseSerializer::from(Response::ok(
                            req_id,
                            Some(ResponseTrsPayload { trs }),
                        )),
                    ),
                    Err(err) => serde_json::to_writer(
                        &stream,
                        &ResponseSerializer::from(err.to_response(req_id)),
                    ),
                }
            }
            bank_protocol::types::Method::AccountBalance => {
                match Self::handler_account_balance(bank, req) {
                    Ok(balance) => serde_json::to_writer(
                        &stream,
                        &ResponseSerializer::from(Response::ok(
                            req_id,
                            Some(ResponseBalancePayload { balance }),
                        )),
                    ),
                    Err(err) => serde_json::to_writer(
                        &stream,
                        &ResponseSerializer::from(err.to_response(req_id)),
                    ),
                }
            }
        };
        
        stream.write_all(b"\n")?;
        Ok(())
    }

    fn handle_create_account(
        bank: Arc<RwLock<Bank<A, T>>>,
        req: Request<Value>,
    ) -> Result<ResponseAccountPayload, ResponseErrorPayload> {
        let payload = match serde_json::from_value::<RequestCreateAccountPayload>(req.payload) {
            Ok(payload) => payload,
            Err(_) => return Err(ResponseErrorPayload::invalid_format()),
        };
        let Account { name, balance, trs } =
            bank.write().unwrap().create_account(payload.account_name)?;
        Ok(ResponseAccountPayload { name, balance, trs })
    }

    fn handle_incr_balance(
        bank: Arc<RwLock<Bank<A, T>>>,
        req: Request<Value>,
    ) -> Result<usize, ResponseErrorPayload> {
        let payload = match serde_json::from_value::<RequestIncrBalancePayload>(req.payload) {
            Ok(payload) => payload,
            Err(_) => return Err(ResponseErrorPayload::invalid_format()),
        };
        Ok(bank
            .write()
            .unwrap()
            .inc_acc_balance(payload.account_name, payload.value)?)
    }

    fn handle_decr_balance(
        bank: Arc<RwLock<Bank<A, T>>>,
        req: Request<Value>,
    ) -> Result<usize, ResponseErrorPayload> {
        let payload = match serde_json::from_value::<RequestDecrBalancePayload>(req.payload) {
            Ok(payload) => payload,
            Err(_) => return Err(ResponseErrorPayload::invalid_format()),
        };
        Ok(bank
            .write()
            .unwrap()
            .decr_acc_balance(payload.account_name, payload.value)?)
    }

    fn handler_make_transaction(
        bank: Arc<RwLock<Bank<A, T>>>,
        req: Request<Value>,
    ) -> Result<usize, ResponseErrorPayload> {
        let payload = match serde_json::from_value::<RequestMakeTransactionPayload>(req.payload) {
            Ok(payload) => payload,
            Err(_) => return Err(ResponseErrorPayload::invalid_format()),
        };

        let tr = bank.write().unwrap().make_transaction(
            payload.account_name,
            payload.account_to_name,
            payload.value,
        )?;
        Ok(tr)
    }

    fn handler_transactions(
        bank: Arc<RwLock<Bank<A, T>>>,
    ) -> Result<Vec<TransactionSerializer>, ResponseErrorPayload> {
        Ok(bank
            .read()
            .unwrap()
            .transactions()?
            .into_iter()
            .map(|tr| TransactionSerializer::from(Tr(tr)))
            .collect())
    }

    fn handler_transaction(
        bank: Arc<RwLock<Bank<A, T>>>,
        req: Request<Value>,
    ) -> Result<TransactionSerializer, ResponseErrorPayload> {
        let payload = match serde_json::from_value::<RequestTransactionByIdPayload>(req.payload) {
            Ok(payload) => payload,
            Err(_) => return Err(ResponseErrorPayload::invalid_format()),
        };
        Ok(TransactionSerializer::from(Tr(bank
            .read()
            .unwrap()
            .transaction_by_id(payload.id)?)))
    }

    fn handler_account_trs(
        bank: Arc<RwLock<Bank<A, T>>>,
        req: Request<Value>,
    ) -> Result<Vec<TransactionSerializer>, ResponseErrorPayload> {
        let payload = match serde_json::from_value::<RequestAccountTransactionsPayload>(req.payload)
        {
            Ok(payload) => payload,
            Err(_) => return Err(ResponseErrorPayload::invalid_format()),
        };
        Ok(bank
            .write()
            .unwrap()
            .account_transactions(payload.account_name)?
            .into_iter()
            .map(|tr| TransactionSerializer::from(Tr(tr)))
            .collect())
    }

    fn handler_account_balance(
        bank: Arc<RwLock<Bank<A, T>>>,
        req: Request<Value>,
    ) -> Result<usize, ResponseErrorPayload> {
        let payload = match serde_json::from_value::<RequestBalancePayload>(req.payload) {
            Ok(payload) => payload,
            Err(_) => return Err(ResponseErrorPayload::invalid_format()),
        };
        Ok(bank.read().unwrap().account_balance(payload.account_name)?)
    }
}
