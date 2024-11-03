use std::{cell::RefCell, io::Write, net::TcpStream, rc::Rc};

use bank_actions::bank::{base::{account::{Account, DecBalanceError, GetAccountError, IncBalanceError, TransferError}, storage::{AccountStorage, Storage, TransactionStorage}}, implements::memory::storage::MemAccountStorage};
use bank_protocol::types::{DecrBalancePayload, MakeTransactionPayload, Request, RequestCreateAccountPayload, RequestIncrBalancePayload, RespCode, Response, ResponseErrorPayload, ResponseOkTransactionPayload, ResponseSerializer};
use serde::{ Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

struct Handler<A: AccountStorage, T: TransactionStorage> {
    bank_storage: Rc<Storage<A, T>>
}




impl <A: AccountStorage, T: TransactionStorage> Handler<A, T> {
    fn handle_msg(&self, req: Request<Value>, resp_pipe: TcpStream)  {
        let err = match req.method {
            bank_protocol::types::Method::CreteAccount => {
                match  self.handle_create_account(req) {
                    Ok(resp) => serde_json::to_writer(resp_pipe, &ResponseSerializer::from(resp)),
                    Err(resp) => serde_json::to_writer(resp_pipe, &ResponseSerializer::from(resp)),
                }
            },
            bank_protocol::types::Method::IncrBalance => {
                match  self.handle_incr_balance(req) {
                    Ok(resp) => serde_json::to_writer(resp_pipe, &ResponseSerializer::from(resp)),
                    Err(resp) => serde_json::to_writer(resp_pipe, &ResponseSerializer::from(resp)),
                }
            },
            bank_protocol::types::Method::DecrBalance => {
                match  self.handle_decr_balance(req) {
                    Ok(resp) => serde_json::to_writer(resp_pipe, &ResponseSerializer::from(resp)),
                    Err(resp) => serde_json::to_writer(resp_pipe, &ResponseSerializer::from(resp)),
                }
            },
            bank_protocol::types::Method::MakeTransaction => todo!(),
            bank_protocol::types::Method::GetTransactions => todo!(),
        };
    }

    fn handle_create_account(&self, req: Request<Value>) -> Result<Response<String>, Response<ResponseErrorPayload>> {
        let payload = match  serde_json::from_value::<RequestCreateAccountPayload>(req.payload) {
            Ok(payload) => payload,
            Err(_) => return Err( Response::<ResponseErrorPayload>::invalid_format(req.id)),
        };

        match Account::new(payload.account_name, self.bank_storage.clone()) {
            Ok(_) => Ok(Response{id: req.id, code: RespCode::OK, payload: None}),
            Err(err) => Err(Response{
                id: req.id,
                code: RespCode::ERR,
                payload: Some(ResponseErrorPayload{error: err.to_string()})
            }),
        }
    }

    fn handle_incr_balance(&self, req: Request<Value>) -> Result<Response<String>, Response<ResponseErrorPayload>>  {
        let payload = match  serde_json::from_value::<RequestIncrBalancePayload>(req.payload) {
            Ok(payload) => payload,
            Err(_) => return Err( Response::<ResponseErrorPayload>::invalid_format(req.id)),
        };

        match Account::account(&payload.account_name, self.bank_storage.clone()) {
            Ok(mut acc) => {
                match acc.inc_balance(payload.value) {
                    Ok(_) => Ok(Response{id: req.id, code: RespCode::OK, payload: None}),
                    Err(IncBalanceError::ZeroInc) => Err(Response{
                        id: req.id,
                        code: RespCode::ERR,
                        payload: Some(ResponseErrorPayload{error: "Zero increment".to_string()})
                    }),
                    Err(IncBalanceError::AccountStorage(_)) => Err(Response{
                        id: req.id,
                        code: RespCode::ERR,
                        payload: Some(ResponseErrorPayload{error: "Account storage error".to_string()})
                    }),
                    Err(IncBalanceError::TransactionStorage(_)) => Err(Response{
                        id: req.id,
                        code: RespCode::ERR,
                        payload: Some(ResponseErrorPayload{error: "Transaction storage error".to_string()})
                    }),
                }

            },
            Err(GetAccountError::AccountNotExists) => {
                Err(Response{
                    id: req.id,
                    code: RespCode::ERR,
                    payload: Some(ResponseErrorPayload{error: "Account not exists".to_string()})
                })
            
            },
            Err(GetAccountError::AccountStorage) => {
                Err(Response{
                    id: req.id,
                    code: RespCode::ERR,
                    payload: Some(ResponseErrorPayload{error: "Account storage error".to_string()})
                })
            }
        }

    }

    fn handle_decr_balance(&self, req: Request<Value>) -> Result<Response<String>, Response<ResponseErrorPayload>>  {
        let payload = match  serde_json::from_value::<DecrBalancePayload>(req.payload) {
            Ok(payload) => payload,
            Err(_) => return Err( Response::<ResponseErrorPayload>::invalid_format(req.id)),
        };

        match Account::account(&payload.account_name, self.bank_storage.clone()) {
            Ok(mut acc) => {
                match acc.decr_balance(payload.value) {
                    Ok(_) => Ok(Response{id: req.id, code: RespCode::OK, payload: None}),
                    Err(DecBalanceError::ZeroDec) => Err(Response{
                        id: req.id,
                        code: RespCode::ERR,
                        payload: Some(ResponseErrorPayload{error: "zero increment".to_string()})
                    }),
                    Err(DecBalanceError::AccountStorage(_)) => Err(Response{
                        id: req.id,
                        code: RespCode::ERR,
                        payload: Some(ResponseErrorPayload{error: "account storage error".to_string()})
                    }),
                    Err(DecBalanceError::TransactionStorage(_)) => Err(Response{
                        id: req.id,
                        code: RespCode::ERR,
                        payload: Some(ResponseErrorPayload{error: "transaction storage error".to_string()})
                    }),
                    Err(DecBalanceError::NotEnoughMoney) => Err(Response{
                        id: req.id,
                        code: RespCode::ERR,
                        payload: Some(ResponseErrorPayload{error: "not enough money".to_string()})
                    }),
                }

            },
            Err(GetAccountError::AccountNotExists) => {
                Err(Response{
                    id: req.id,
                    code: RespCode::ERR,
                    payload: Some(ResponseErrorPayload{error: "Account not exists".to_string()})
                })
            
            },
            Err(GetAccountError::AccountStorage) => {
                Err(Response{
                    id: req.id,
                    code: RespCode::ERR,
                    payload: Some(ResponseErrorPayload{error: "Account storage error".to_string()})
                })
            }
        }

    }

    fn handle_make_transaction(&self, req: Request<Value>) -> Result<Response<ResponseOkTransactionPayload>, Response<ResponseErrorPayload>> {
        let payload = match  serde_json::from_value::<MakeTransactionPayload>(req.payload) {
            Ok(payload) => payload,
            Err(_) => return Err( Response::<ResponseErrorPayload>::invalid_format(req.id)),
        };

        // get accs
        let mut from_acc = Handler::get_account(req.id, &payload.account_name, self.bank_storage.clone())?;
        let mut to_acc = Handler::get_account(req.id, &payload.account_to_name, self.bank_storage.clone())?;

        // make transaction
        match from_acc.make_transaction(payload.value, &mut to_acc, payload.fee_amount) {
            Ok(tr_id) => Ok(Response{id: req.id, code: RespCode::OK, payload: Some(ResponseOkTransactionPayload{transaction_id: tr_id})}),
            Err(TransferError::NotEnoughBalance) => Err(Response::new(req.id, "not enough money".to_owned())),
            Err(TransferError::ZeroTransfer) => Err(Response::new(req.id, "empty transfer".to_owned())),
            Err(Error::CreateTransaction(_)) => Err(Response::new(req.id, "create transaction error".to_owned())),
            Err(TransferError::UpdateAccount(_)) => Err(Response::new(req.id, "account storage error".to_owned())),
            Err(TransferError::GetFeeAccount(_)) => Err(Response::new(req.id, "fee collision".to_owned()))
        }
        






    }

    fn get_account(req_id: Uuid, account_name: &String, storage:  Rc<Storage<A, T>>) -> Result<Account<A, T>, Response<ResponseErrorPayload>> {
        match Account::account(account_name, storage.clone()) {
            Ok(acc) => {Ok(acc)},
            Err(GetAccountError::AccountNotExists) => {
                Err(Response{
                    id: req_id,
                    code: RespCode::ERR,
                    payload: Some(ResponseErrorPayload{error: "Account not exists".to_string()})
                })
            
            },
            Err(GetAccountError::AccountStorage) => {
                Err(Response{
                    id: req_id,
                    code: RespCode::ERR,
                    payload: Some(ResponseErrorPayload{error: "Account storage error".to_string()})
                })
            }
        }



    }
}
