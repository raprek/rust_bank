use std::{sync::Arc};

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
use serde_json::Value;
use tokio::{sync::mpsc::Receiver, task::JoinHandle};
use tokio::sync::{Mutex, RwLock};

use crate::{handler, server::HandleItem};

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
    pub fn run(mut handler: Self) -> JoinHandle<()>{
        println!("Handler started");
        tokio::spawn(async move {
            loop {
                let h_item = handler.recv_chan.recv().await.unwrap();
                println!("New msg in handler {:?}", h_item.req);
                let bank = handler.bank.clone();
                tokio::spawn(async move {
                    match Self::handle_msg(bank, h_item.req.clone(), h_item.resp_sender).await {
                        Ok(_) => println!("Item suc handled. Req: {:?}", h_item.req),
                        Err(_) => println!("Error handling item. Req: {:?}", h_item.req),
                    }
                });
            }
        })
    }

    pub async fn handle_msg(
        bank: Arc<RwLock<Bank<A, T>>>,
        req: Request<Value>,
        resp_sender: tokio::sync::mpsc::Sender<String>,
    ) -> Result<(), std::io::Error> {
        let req_id = req.id;
        let res = match req.method {
            bank_protocol::types::Method::CreteAccount => {
                match Self::handle_create_account(bank, req).await {
                    Ok(acc) => {
                        let payload = acc;
                        serde_json::to_string(&ResponseSerializer::from(Response::<
                            ResponseAccountPayload,
                        >::ok(
                            req_id, Some(payload)
                        )))?
                    }
                    Err(err) => {
                        serde_json::to_string(&ResponseSerializer::from(err.to_response(req_id)))?
                    }
                }
            }
            bank_protocol::types::Method::IncrBalance => {
                match Self::handle_incr_balance(bank, req).await {
                    Ok(id) => serde_json::to_string(&ResponseSerializer::from(Response::ok(
                        req_id,
                        Some(ResponseShortTrPayload { id }),
                    )))?,
                    Err(err) => {
                        serde_json::to_string(&ResponseSerializer::from(err.to_response(req_id)))?
                    }
                }
            }
            bank_protocol::types::Method::DecrBalance => {
                match Self::handle_decr_balance(bank, req).await {
                    Ok(id) => serde_json::to_string(&ResponseSerializer::from(Response::ok(
                        req_id,
                        Some(ResponseShortTrPayload { id }),
                    )))?,
                    Err(err) => {
                        serde_json::to_string(&ResponseSerializer::from(err.to_response(req_id)))?
                    }
                }
            }
            bank_protocol::types::Method::MakeTransaction => {
                match Self::handler_make_transaction(bank, req).await {
                    Ok(id) => serde_json::to_string(&ResponseSerializer::from(Response::ok(
                        req_id,
                        Some(ResponseShortTrPayload { id }),
                    )))?,
                    Err(err) => {
                        serde_json::to_string(&ResponseSerializer::from(err.to_response(req_id)))?
                    }
                }
            }
            bank_protocol::types::Method::Transactions => {
                match Self::handler_transactions(bank).await {
                    Ok(trs) => serde_json::to_string(&ResponseSerializer::from(Response::ok(
                        req_id,
                        Some(ResponseTrsPayload { trs }),
                    )))?,
                    Err(err) => {
                        serde_json::to_string(&ResponseSerializer::from(err.to_response(req_id)))?
                    }
                }
            }
            bank_protocol::types::Method::Transaction => {
                match Self::handler_transaction(bank, req).await {
                    Ok(tr) => serde_json::to_string(&ResponseSerializer::from(Response::ok(
                        req_id,
                        Some(ResponseTrPayload { tr }),
                    )))?,
                    Err(err) => {
                        serde_json::to_string(&ResponseSerializer::from(err.to_response(req_id)))?
                    }
                }
            }
            bank_protocol::types::Method::AccountTransactions => {
                match Self::handler_account_trs(bank, req).await {
                    Ok(trs) => serde_json::to_string(&ResponseSerializer::from(Response::ok(
                        req_id,
                        Some(ResponseTrsPayload { trs }),
                    )))?,
                    Err(err) => {
                        serde_json::to_string(&ResponseSerializer::from(err.to_response(req_id)))?
                    }
                }
            }
            bank_protocol::types::Method::AccountBalance => {
                match Self::handler_account_balance(bank, req).await {
                    Ok(balance) => serde_json::to_string(&ResponseSerializer::from(Response::ok(
                        req_id,
                        Some(ResponseBalancePayload { balance }),
                    )))?,
                    Err(err) => {
                        serde_json::to_string(&ResponseSerializer::from(err.to_response(req_id)))?
                    }
                }
            }
        };
        resp_sender.send(res).await.unwrap();
        println!("Finish write response");
        Ok(())
    }

    async fn handle_create_account(
        bank: Arc<RwLock<Bank<A, T>>>,
        req: Request<Value>,
    ) -> Result<ResponseAccountPayload, ResponseErrorPayload> {
        let payload = match serde_json::from_value::<RequestCreateAccountPayload>(req.payload) {
            Ok(payload) => payload,
            Err(_) => return Err(ResponseErrorPayload::invalid_format()),
        };
        let Account { name, balance, trs } =
            bank.write().await.create_account(payload.account_name)?;
        Ok(ResponseAccountPayload { name, balance, trs })
    }

    async fn handle_incr_balance(
        bank: Arc<RwLock<Bank<A, T>>>,
        req: Request<Value>,
    ) -> Result<usize, ResponseErrorPayload> {
        let payload = match serde_json::from_value::<RequestIncrBalancePayload>(req.payload) {
            Ok(payload) => payload,
            Err(_) => return Err(ResponseErrorPayload::invalid_format()),
        };
        Ok(bank
            .write()
            .await
            .inc_acc_balance(payload.account_name, payload.value)?)
    }

    async fn handle_decr_balance(
        bank: Arc<RwLock<Bank<A, T>>>,
        req: Request<Value>,
    ) -> Result<usize, ResponseErrorPayload> {
        let payload = match serde_json::from_value::<RequestDecrBalancePayload>(req.payload) {
            Ok(payload) => payload,
            Err(_) => return Err(ResponseErrorPayload::invalid_format()),
        };
        Ok(bank
            .write()
            .await
            .decr_acc_balance(payload.account_name, payload.value)?)
    }

    async fn handler_make_transaction(
        bank: Arc<RwLock<Bank<A, T>>>,
        req: Request<Value>,
    ) -> Result<usize, ResponseErrorPayload> {
        let payload = match serde_json::from_value::<RequestMakeTransactionPayload>(req.payload) {
            Ok(payload) => payload,
            Err(_) => return Err(ResponseErrorPayload::invalid_format()),
        };

        let tr = bank.write().await.make_transaction(
            payload.account_name,
            payload.account_to_name,
            payload.value,
        )?;
        Ok(tr)
    }

    async fn handler_transactions(
        bank: Arc<RwLock<Bank<A, T>>>,
    ) -> Result<Vec<TransactionSerializer>, ResponseErrorPayload> {
        Ok(bank
            .read()
            .await
            .transactions()?
            .into_iter()
            .map(|tr| TransactionSerializer::from(Tr(tr)))
            .collect())
    }

    async fn handler_transaction(
        bank: Arc<RwLock<Bank<A, T>>>,
        req: Request<Value>,
    ) -> Result<TransactionSerializer, ResponseErrorPayload> {
        let payload = match serde_json::from_value::<RequestTransactionByIdPayload>(req.payload) {
            Ok(payload) => payload,
            Err(_) => return Err(ResponseErrorPayload::invalid_format()),
        };
        Ok(TransactionSerializer::from(Tr(bank
            .read()
            .await
            .transaction_by_id(payload.id)?)))
    }

    async fn handler_account_trs(
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
            .await
            .account_transactions(payload.account_name)?
            .into_iter()
            .map(|tr| TransactionSerializer::from(Tr(tr)))
            .collect())
    }

    async fn handler_account_balance(
        bank: Arc<RwLock<Bank<A, T>>>,
        req: Request<Value>,
    ) -> Result<usize, ResponseErrorPayload> {
        let payload = match serde_json::from_value::<RequestBalancePayload>(req.payload) {
            Ok(payload) => payload,
            Err(_) => return Err(ResponseErrorPayload::invalid_format()),
        };
        Ok(bank.read().await.account_balance(payload.account_name)?)
    }
}
