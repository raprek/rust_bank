use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub enum TransactionActionSerializer {
    Registration,
    Add(usize),
    Withdraw(usize),
    Transfer {
        to: String, // account id
        value: usize,
        fee: usize,
    },
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TransactionSerializer {
    pub id: usize,
    pub action: TransactionActionSerializer,
    pub account_name: String,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct AccountSerializer {
    pub balance: usize,
    pub name: String,
    pub trs: Vec<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum Method {
    CreteAccount,
    IncrBalance,
    DecrBalance,
    MakeTransaction,
    Transaction,
    Transactions,
    AccountTransactions,
    AccountBalance,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum RespCode {
    OK,
    ERR,
}

#[derive(Debug, Clone)]
pub struct Request<P: Serialize> {
    pub id: Uuid,
    pub method: Method,
    pub payload: P,
}

#[derive(Debug)]
pub struct Response<P: Serialize> {
    pub id: Uuid,
    pub code: RespCode,
    pub payload: Option<P>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ResponseSerializer<P: Serialize> {
    pub id: String,
    pub code: RespCode,
    pub payload: Option<P>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RequestSerializer<P: Serialize> {
    pub id: String,
    pub method: Method,
    pub payload: P,
}

#[derive(Serialize, Deserialize)]
pub struct ResponseErrorPayload {
    pub error: String,
}

#[derive(Serialize, Deserialize)]
pub struct ResponseAccountPayload {
    pub balance: usize,
    pub name: String,
    pub trs: Vec<usize>,
}

#[derive(Serialize, Deserialize)]
pub struct ResponseShortTrPayload {
    pub id: usize,
}

#[derive(Serialize, Deserialize)]
pub struct ResponseTrsPayload {
    pub trs: Vec<TransactionSerializer>,
}

#[derive(Serialize, Deserialize)]
pub struct ResponseTrPayload {
    pub tr: TransactionSerializer,
}

#[derive(Serialize, Deserialize)]
pub struct ResponseBalancePayload {
    pub balance: usize,
}

#[derive(Serialize, Deserialize)]
pub struct RequestCreateAccountPayload {
    pub account_name: String,
}

#[derive(Serialize, Deserialize)]
pub struct RequestIncrBalancePayload {
    pub account_name: String,
    pub value: usize,
}

#[derive(Serialize, Deserialize)]
pub struct RequestDecrBalancePayload {
    pub account_name: String,
    pub value: usize,
}

#[derive(Serialize, Deserialize)]
pub struct RequestMakeTransactionPayload {
    pub account_name: String,
    pub account_to_name: String,
    pub value: usize,
}

// todo delete in future
#[derive(Serialize, Deserialize)]
pub struct RequestTransactionsPayload {}

#[derive(Serialize, Deserialize)]
pub struct RequestTransactionByIdPayload {
    pub id: usize,
}

#[derive(Serialize, Deserialize)]
pub struct RequestAccountTransactionsPayload {
    pub account_name: String,
}

#[derive(Serialize, Deserialize)]
pub struct RequestBalancePayload {
    pub account_name: String,
}

impl Response<ResponseErrorPayload> {
    pub fn new(req_id: Uuid, error: String) -> Self {
        Response {
            id: req_id,
            code: RespCode::ERR,
            payload: Some(ResponseErrorPayload { error }),
        }
    }
}

impl ResponseErrorPayload {
    pub fn invalid_format() -> ResponseErrorPayload {
        ResponseErrorPayload {
            error: "InvalidFormat".to_string(),
        }
    }

    pub fn to_response(self, req_id: Uuid) -> Response<Self> {
        Response {
            id: req_id,
            code: RespCode::ERR,
            payload: Some(self),
        }
    }
}

impl<E: ToString> From<E> for ResponseErrorPayload {
    fn from(value: E) -> Self {
        ResponseErrorPayload {
            error: value.to_string(),
        }
    }
}

impl<P: Serialize> Response<P> {
    pub fn ok(req_id: Uuid, payload: Option<P>) -> Self {
        Response {
            id: req_id,
            code: RespCode::OK,
            payload,
        }
    }

    pub fn err(req_id: Uuid, payload: Option<P>) -> Self {
        Response {
            id: req_id,
            code: RespCode::ERR,
            payload,
        }
    }
}

impl<P: Serialize> TryFrom<ResponseSerializer<P>> for Response<P> {
    type Error = uuid::Error;

    fn try_from(value: ResponseSerializer<P>) -> Result<Self, Self::Error> {
        let uuid = match Uuid::parse_str(value.id.as_str()) {
            Ok(uuid) => uuid,
            Err(err) => return Err(err),
        };
        Ok(Response {
            id: uuid,
            payload: value.payload,
            code: value.code,
        })
    }
}

impl<P: Serialize> From<Request<P>> for RequestSerializer<P> {
    fn from(value: Request<P>) -> Self {
        RequestSerializer {
            id: value.id.to_string(),
            method: value.method,
            payload: value.payload,
        }
    }
}

impl<P: Serialize> TryFrom<RequestSerializer<P>> for Request<P> {
    type Error = uuid::Error;

    fn try_from(value: RequestSerializer<P>) -> Result<Self, Self::Error> {
        let uuid = match Uuid::parse_str(value.id.as_str()) {
            Ok(uuid) => uuid,
            Err(err) => return Err(err),
        };
        Ok(Request {
            id: uuid,
            method: value.method,
            payload: value.payload,
        })
    }
}

impl<P: Serialize> From<Response<P>> for ResponseSerializer<P> {
    fn from(value: Response<P>) -> Self {
        ResponseSerializer {
            id: value.id.to_string(),
            payload: value.payload,
            code: value.code,
        }
    }
}

impl<P: Serialize> Request<P> {
    pub fn new(method: Method, payload: P) -> Self {
        Self {
            id: Uuid::new_v4(),
            method,
            payload,
        }
    }
}

impl From<ResponseAccountPayload> for AccountSerializer {
    fn from(value: ResponseAccountPayload) -> Self {
        AccountSerializer {
            balance: value.balance,
            name: value.name,
            trs: value.trs,
        }
    }
}

impl From<AccountSerializer> for ResponseAccountPayload {
    fn from(value: AccountSerializer) -> Self {
        ResponseAccountPayload {
            balance: value.balance,
            name: value.name,
            trs: value.trs,
        }
    }
}
