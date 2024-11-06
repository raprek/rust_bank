use std::net::TcpStream;


use bank_core::bank::{
    storage::{AccountStorage, TransactionStorage}, transactions::Transaction, Bank
};
use bank_protocol::types::{Request, RequestAccountTransactionsPayload, RequestCreateAccountPayload, RequestDecrBalancePayload, RequestIncrBalancePayload, RequestMakeTransactionPayload, RequestTransactionByIdPayload, RequestTransactionsPayload, RespCode, Response, ResponseDecrBalancePayload, ResponseErrorPayload, ResponseIncrBalancePayload, ResponseMakeTrPayload, ResponseOkTransactionPayload, ResponseSerializer, ResponseTrPayload, ResponseTrsPayload};
use serde_json::Value;

pub struct Handler<A: AccountStorage + Default, T: TransactionStorage + Default> {
    bank: Bank<A, T>,
}

impl <A: AccountStorage + Default, T: TransactionStorage + Default> Handler<A, T> {
    pub fn new(bank: Bank<A, T>) -> Self {
        Self{bank}
    }
    
    pub fn handle_msg(&mut self, req: Request<Value>, resp_pipe: TcpStream) {
        let req_id = req.id;
        let _ = match req.method {
            bank_protocol::types::Method::CreteAccount => {
                match  self.handle_create_account(req) {
                    Ok(_) => serde_json::to_writer(resp_pipe, &ResponseSerializer::from(Response::<()>::ok(req_id, None))),
                    Err(err) => serde_json::to_writer(resp_pipe, &ResponseSerializer::from(err.to_response(req_id))),
                }
            },
            bank_protocol::types::Method::IncrBalance => {
                match  self.handle_incr_balance(req) {
                    Ok(id) => serde_json::to_writer(resp_pipe, &ResponseSerializer::from(Response::ok(req_id, Some(ResponseIncrBalancePayload{id})))),
                    Err(err) => serde_json::to_writer(resp_pipe, &ResponseSerializer::from(err.to_response(req_id))),
                }
            },
            bank_protocol::types::Method::DecrBalance => {
                match  self.handle_decr_balance(req) {
                    Ok(id) => serde_json::to_writer(resp_pipe, &ResponseSerializer::from(Response::ok(req_id, Some(ResponseDecrBalancePayload{id})))),
                    Err(err) => serde_json::to_writer(resp_pipe, &ResponseSerializer::from(err.to_response(req_id))),
                }
            },
            bank_protocol::types::Method::MakeTransaction => {
                match  self.handler_make_transaction(req) {
                    Ok(id) => serde_json::to_writer(resp_pipe, &ResponseSerializer::from(Response::ok(req_id, Some(ResponseMakeTrPayload{id})))),
                    Err(err) => serde_json::to_writer(resp_pipe, &ResponseSerializer::from(err.to_response(req_id))),
                }
            },
            bank_protocol::types::Method::Transactions => {
                match  self.handler_transactions() {
                    Ok(trs) => serde_json::to_writer(resp_pipe, &ResponseSerializer::from(Response::ok(req_id, Some(ResponseTrsPayload{trs})))),
                    Err(err) => serde_json::to_writer(resp_pipe, &ResponseSerializer::from(err.to_response(req_id))),
                }
            },
            bank_protocol::types::Method::Transaction => {
                match  self.handler_transaction(req) {
                    Ok(tr) => serde_json::to_writer(resp_pipe, &ResponseSerializer::from(Response::ok(req_id, Some(ResponseTrPayload{tr})))),
                    Err(err) => serde_json::to_writer(resp_pipe, &ResponseSerializer::from(err.to_response(req_id))),
                }
            },
            bank_protocol::types::Method::AccountTransactions => {
                match  self.handler_account_trs(req) {
                    Ok(trs) => serde_json::to_writer(resp_pipe, &ResponseSerializer::from(Response::ok(req_id, Some(ResponseTrsPayload{trs})))),
                    Err(err) => serde_json::to_writer(resp_pipe, &ResponseSerializer::from(err.to_response(req_id))),
                }
            },
        };
    }

    fn handle_create_account(&mut self, req: Request<Value>) -> Result<(), ResponseErrorPayload> {
        let payload = match  serde_json::from_value::<RequestCreateAccountPayload>(req.payload) {
            Ok(payload) => payload,
            Err(_) => return Err(ResponseErrorPayload::invalid_format()),
        };
        self.bank.create_account(payload.account_name)?;
        Ok(())
    }

    fn handle_incr_balance(&mut self, req: Request<Value>) -> Result<usize, ResponseErrorPayload>  {
        let payload = match  serde_json::from_value::<RequestIncrBalancePayload>(req.payload) {
            Ok(payload) => payload,
            Err(_) => return Err(ResponseErrorPayload::invalid_format()),
        };
        let mut acc = self.bank.get_acc(payload.account_name)?;
        Ok(self.bank.inc_acc_balance(&mut acc, payload.value)?)
    }

    fn handle_decr_balance(&mut self, req: Request<Value>) -> Result<usize, ResponseErrorPayload>  {
        let payload = match  serde_json::from_value::<RequestDecrBalancePayload>(req.payload) {
            Ok(payload) => payload,
            Err(_) => return Err(ResponseErrorPayload::invalid_format()),
        };
        let mut acc = self.bank.get_acc(payload.account_name)?;
        Ok(self.bank.decr_acc_balance(&mut acc, payload.value)?)
    }

    fn handler_make_transaction(&mut self, req: Request<Value>) -> Result<usize, ResponseErrorPayload> {
        let payload = match  serde_json::from_value::<RequestMakeTransactionPayload>(req.payload) {
            Ok(payload) => payload,
            Err(_) => return Err(ResponseErrorPayload::invalid_format()),
        };

        let mut acc_from = self.bank.get_acc(payload.account_name)?;
        let mut acc_to = self.bank.get_acc(payload.account_to_name)?;
        let tr = self.bank.make_transaction(&mut acc_from, &mut acc_to, payload.value)?;
        Ok(tr)
    }

    fn handler_transactions(&mut self) -> Result<Vec<Transaction>, ResponseErrorPayload> {
        Ok(self.bank.transactions()?)
    }

    fn handler_transaction(&mut self, req: Request<Value>) -> Result<Transaction, ResponseErrorPayload> {
        let payload = match  serde_json::from_value::<RequestTransactionByIdPayload>(req.payload) {
            Ok(payload) => payload,
            Err(_) => return Err(ResponseErrorPayload::invalid_format()),
        };
        Ok(self.bank.transaction_by_id(payload.id)?)
    }

    fn handler_account_trs(&mut self, req: Request<Value>) -> Result<Vec<Transaction>, ResponseErrorPayload> {
        let payload = match  serde_json::from_value::<RequestAccountTransactionsPayload>(req.payload) {
            Ok(payload) => payload,
            Err(_) => return Err(ResponseErrorPayload::invalid_format()),
        };
        Ok(self.bank.account_transactions(payload.account_name)?)
    }

    
}
