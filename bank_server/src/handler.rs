use std::{io::Write, net::TcpStream};

use bank_core::bank::{
    storage::{AccountStorage, TransactionAction, TransactionStorage},
    Account, Bank, Transaction,
};
use bank_protocol::types::{
    Request, RequestAccountTransactionsPayload, RequestCreateAccountPayload,
    RequestDecrBalancePayload, RequestIncrBalancePayload, RequestMakeTransactionPayload,
    RequestTransactionByIdPayload, Response, ResponseAccountPayload, ResponseErrorPayload,
    ResponseSerializer, ResponseShortTrPayload, ResponseTrPayload, ResponseTrsPayload,
    TransactionActionSerializer, TransactionSerializer,
};
use serde_json::Value;

pub struct Handler<A: AccountStorage + Default, T: TransactionStorage + Default> {
    bank: Bank<A, T>,
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

impl<A: AccountStorage + Default, T: TransactionStorage + Default> Handler<A, T> {
    pub fn new(bank: Bank<A, T>) -> Self {
        Self { bank }
    }

    pub fn handle_msg(
        &mut self,
        req: Request<Value>,
        mut stream: TcpStream,
    ) -> Result<(), std::io::Error> {
        let req_id = req.id;
        let _ = match req.method {
            bank_protocol::types::Method::CreteAccount => match self.handle_create_account(req) {
                Ok(acc) => {
                    let payload = ResponseAccountPayload::from(acc);
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
            },
            bank_protocol::types::Method::IncrBalance => match self.handle_incr_balance(req) {
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
            bank_protocol::types::Method::DecrBalance => match self.handle_decr_balance(req) {
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
                match self.handler_make_transaction(req) {
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
            bank_protocol::types::Method::Transactions => match self.handler_transactions() {
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
            bank_protocol::types::Method::Transaction => match self.handler_transaction(req) {
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
                match self.handler_account_trs(req) {
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
        };
        stream.write(b"\n")?;
        stream.flush()?;
        Ok(())
    }

    fn handle_create_account(
        &mut self,
        req: Request<Value>,
    ) -> Result<ResponseAccountPayload, ResponseErrorPayload> {
        let payload = match serde_json::from_value::<RequestCreateAccountPayload>(req.payload) {
            Ok(payload) => payload,
            Err(_) => return Err(ResponseErrorPayload::invalid_format()),
        };
        let Account { name, balance, trs } = self.bank.create_account(payload.account_name)?;
        Ok(ResponseAccountPayload { name, balance, trs })
    }

    fn handle_incr_balance(&mut self, req: Request<Value>) -> Result<usize, ResponseErrorPayload> {
        let payload = match serde_json::from_value::<RequestIncrBalancePayload>(req.payload) {
            Ok(payload) => payload,
            Err(_) => return Err(ResponseErrorPayload::invalid_format()),
        };
        Ok(self
            .bank
            .inc_acc_balance(payload.account_name, payload.value)?)
    }

    fn handle_decr_balance(&mut self, req: Request<Value>) -> Result<usize, ResponseErrorPayload> {
        let payload = match serde_json::from_value::<RequestDecrBalancePayload>(req.payload) {
            Ok(payload) => payload,
            Err(_) => return Err(ResponseErrorPayload::invalid_format()),
        };
        Ok(self
            .bank
            .decr_acc_balance(payload.account_name, payload.value)?)
    }

    fn handler_make_transaction(
        &mut self,
        req: Request<Value>,
    ) -> Result<usize, ResponseErrorPayload> {
        let payload = match serde_json::from_value::<RequestMakeTransactionPayload>(req.payload) {
            Ok(payload) => payload,
            Err(_) => return Err(ResponseErrorPayload::invalid_format()),
        };

        let tr = self.bank.make_transaction(
            payload.account_name,
            payload.account_to_name,
            payload.value,
        )?;
        Ok(tr)
    }

    fn handler_transactions(&mut self) -> Result<Vec<TransactionSerializer>, ResponseErrorPayload> {
        Ok(self
            .bank
            .transactions()?
            .into_iter()
            .map(|tr| TransactionSerializer::from(Tr(tr)))
            .collect())
    }

    fn handler_transaction(
        &mut self,
        req: Request<Value>,
    ) -> Result<TransactionSerializer, ResponseErrorPayload> {
        let payload = match serde_json::from_value::<RequestTransactionByIdPayload>(req.payload) {
            Ok(payload) => payload,
            Err(_) => return Err(ResponseErrorPayload::invalid_format()),
        };
        Ok(TransactionSerializer::from(Tr(self
            .bank
            .transaction_by_id(payload.id)?)))
    }

    fn handler_account_trs(
        &mut self,
        req: Request<Value>,
    ) -> Result<Vec<TransactionSerializer>, ResponseErrorPayload> {
        let payload = match serde_json::from_value::<RequestAccountTransactionsPayload>(req.payload)
        {
            Ok(payload) => payload,
            Err(_) => return Err(ResponseErrorPayload::invalid_format()),
        };
        Ok(self
            .bank
            .account_transactions(payload.account_name)?
            .into_iter()
            .map(|tr| TransactionSerializer::from(Tr(tr)))
            .collect())
    }
}
