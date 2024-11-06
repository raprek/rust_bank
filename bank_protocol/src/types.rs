use bank_core::bank::transactions::Transaction;
use serde::{ Deserialize, Serialize};
use uuid::Uuid;


#[derive(Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum Method {
    CreteAccount,
    IncrBalance,
    DecrBalance,
    MakeTransaction,
    Transactions,
    Transaction,
    AccountTransactions
}

#[derive(Serialize, Deserialize)]
pub enum RespCode {
    OK,
    ERR,
}

pub struct Request<P: Serialize> {
    pub id: Uuid,
    pub method: Method,
    pub payload: P
}


pub struct Response<P: Serialize> {
    pub id: Uuid,
    pub code: RespCode,
    pub payload: Option<P>
}

#[derive(Serialize, Deserialize)]
pub struct ResponseSerializer<P: Serialize> {
    id: String,
    code: RespCode,
    payload: Option<P>
}


#[derive(Serialize, Deserialize)]
pub struct RequestSerializer<P: Serialize> {
    pub id: String,
    pub method: Method,
    pub payload: P

}


#[derive(Serialize, Deserialize)]
pub struct ResponseErrorPayload {
    pub error: String
}

#[derive(Serialize, Deserialize)]
pub struct ResponseIncrBalancePayload {
    pub id: usize
}

#[derive(Serialize, Deserialize)]
pub struct ResponseDecrBalancePayload {
    pub id: usize
}

#[derive(Serialize, Deserialize)]
pub struct ResponseMakeTrPayload {
    pub id: usize
}

#[derive(Serialize, Deserialize)]
pub struct ResponseTrsPayload {
    pub trs: Vec<Transaction>
}

#[derive(Serialize, Deserialize)]
pub struct ResponseTrPayload {
    pub tr: Transaction
}

#[derive(Serialize, Deserialize)]
pub struct ResponseOkTransactionPayload {
    pub transaction_id: usize
}



#[derive(Deserialize)]
pub struct RequestCreateAccountPayload {
    pub account_name: String
}

#[derive(Deserialize)]
pub struct RequestIncrBalancePayload {
    pub account_name: String,
    pub value: usize
}

#[derive(Serialize, Deserialize)]
pub struct RequestDecrBalancePayload {
    pub account_name: String,
    pub value: usize
}

#[derive(Serialize, Deserialize)]
pub struct RequestMakeTransactionPayload {
    pub account_name: String,
    pub account_to_name: String,
    pub value: usize,
    pub fee_amount: Option<usize>
}

#[derive(Serialize, Deserialize)]
pub struct RequestAccountTransactionsPayload  {
    pub account_name: String,
}

// todo delete in future
#[derive(Serialize, Deserialize)]
pub struct RequestTransactionsPayload {
}

#[derive(Serialize, Deserialize)]
pub struct RequestTransactionByIdPayload {
    pub id: usize,
}


impl Response<ResponseErrorPayload> {
    pub fn new(req_id: Uuid, error: String) -> Self {
        Response{
            id: req_id,
            code: RespCode::ERR,
            payload: Some(ResponseErrorPayload{error: error})
        }

    }
}

impl ResponseErrorPayload {
    pub fn invalid_format()  -> ResponseErrorPayload {
        ResponseErrorPayload{error: "InvalidFormat".to_string()}
    }

    pub fn to_response(self, req_id: Uuid) -> Response<Self> {
        Response{
            id: req_id,
            code: RespCode::ERR,
            payload: Some(self)
        }
    }
}

impl <E: ToString>From<E> for ResponseErrorPayload {
    fn from(value: E) -> Self {
        ResponseErrorPayload{error: value.to_string()}
    }
}

impl <P: Serialize>Response<P> {
    pub fn ok(req_id: Uuid, payload: Option<P>) -> Self {
        Response{
            id: req_id,
            code: RespCode::OK,
            payload: payload
        }

    }

    pub fn err(req_id: Uuid, payload: Option<P>) -> Self {
        Response{
            id: req_id,
            code: RespCode::ERR,
            payload: payload
        }

    }
}

impl <P: Serialize>TryFrom<ResponseSerializer<P>> for  Response<P>{
    type Error = String;

    fn try_from(value: ResponseSerializer<P>) -> Result<Self, Self::Error> {
        let uuid = match  Uuid::parse_str(value.id.as_str()){
            Ok(uuid) => uuid,
            Err(err) => return Err(err.to_string()),
        };
        Ok(Response{
            id: uuid,
            payload: value.payload,
            code: value.code
        })
    }
}

impl <P: Serialize>From<Request<P>> for RequestSerializer<P> {
    fn from(value: Request<P>) -> Self {
        RequestSerializer{
            id: value.id.to_string(),
            method: value.method,
            payload: value.payload
        }
    }
}


impl <P: Serialize>TryFrom<RequestSerializer<P>> for  Request<P>{
    type Error = String;

    fn try_from(value: RequestSerializer<P>) -> Result<Self, Self::Error> {
        let uuid = match  Uuid::parse_str(value.id.as_str()){
            Ok(uuid) => uuid,
            Err(err) => return Err(err.to_string()),
        };
        Ok(Request{
            id: uuid,
            method: value.method,
            payload: value.payload
        })
    }
}

impl <P: Serialize>From<Response<P>> for ResponseSerializer<P> {
    fn from(value: Response<P>) -> Self {
        ResponseSerializer{
            id: value.id.to_string(),
            payload: value.payload,
            code: value.code
        }
    }
}



