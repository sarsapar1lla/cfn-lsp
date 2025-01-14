// reference: https://www.jsonrpc.org/specification
use method::diagnostic;
use method::initialise;
use method::NotificationMethod;
use method::RequestMethod;
use serde::{Deserialize, Serialize};

pub mod method;

#[derive(Debug)]
#[cfg_attr(test, derive(PartialEq, Eq))]
pub struct ContentType {
    content_type: String,
    charset: String,
}

impl ContentType {
    pub fn new(content_type: &str, charset: &str) -> Self {
        Self {
            content_type: content_type.into(),
            charset: charset.into(),
        }
    }
}

impl Default for ContentType {
    fn default() -> Self {
        Self {
            content_type: "application/vscode-jsonrpc".into(),
            charset: "utf-8".into(),
        }
    }
}

#[derive(Debug)]
#[cfg_attr(test, derive(PartialEq, Eq))]
pub struct Headers {
    content_length: usize,
    content_type: ContentType,
}

impl Headers {
    pub fn new(content_length: usize, content_type: ContentType) -> Self {
        Self {
            content_length,
            content_type,
        }
    }

    pub fn content_length(&self) -> &usize {
        &self.content_length
    }
}

#[derive(Debug)]
#[cfg_attr(test, derive(PartialEq, Eq))]
pub enum Header {
    ContentLength(u32),
    ContentType {
        content_type: String,
        charset: String,
    },
}

#[derive(Debug, Deserialize, Serialize)]
#[cfg_attr(test, derive(PartialEq, Eq))]
pub enum Version {
    #[serde(rename = "2.0")]
    V2,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[cfg_attr(test, derive(PartialEq, Eq))]
#[serde(untagged)]
pub enum RequestId {
    String(String),
    Number(u32),
    Null,
}

#[derive(Debug, Deserialize)]
#[cfg_attr(test, derive(PartialEq, Eq))]
#[serde(untagged)]
pub enum Message {
    Request(Request),
    BatchRequest(Vec<Request>),
    Notification(Notification),
}

#[derive(Debug, Deserialize)]
#[cfg_attr(test, derive(PartialEq, Eq))]
pub struct Request {
    jsonrpc: Version,
    #[serde(flatten)]
    method: RequestMethod,
    id: RequestId,
}

impl Request {
    pub fn method(&self) -> &RequestMethod {
        &self.method
    }

    pub fn id(&self) -> &RequestId {
        &self.id
    }
}

#[derive(Debug, Deserialize)]
#[cfg_attr(test, derive(PartialEq, Eq))]
pub struct Notification {
    jsonrpc: Version,
    #[serde(flatten)]
    method: NotificationMethod,
}

impl Notification {
    pub fn method(&self) -> &NotificationMethod {
        &self.method
    }
}

#[derive(Debug, Serialize)]
#[serde(untagged)]
pub enum ResponseResult {
    Initialise(initialise::Result),
    TextDocumentDiagnostic(diagnostic::Result),
    Null,
}

#[derive(Debug, Serialize)]
#[serde(untagged)]
pub enum Response {
    Success {
        jsonrpc: Version,
        result: ResponseResult,
        id: RequestId,
    },
    Error {
        jsonrpc: Version,
        error: Error,
        id: RequestId,
    },
    Batch(Vec<Response>),
}

impl Response {
    pub fn success(id: &RequestId, result: ResponseResult) -> Self {
        Response::Success {
            jsonrpc: Version::V2,
            result,
            id: id.clone(),
        }
    }

    pub fn error(id: &RequestId, error: Error) -> Self {
        Response::Error {
            jsonrpc: Version::V2,
            error,
            id: id.clone(),
        }
    }
}

#[derive(Debug, Serialize)]
pub struct Error {
    code: ErrorCode,
    message: String,
    data: Option<serde_json::Value>,
}

impl Error {
    pub fn new(code: ErrorCode, message: &str, data: Option<serde_json::Value>) -> Self {
        Error {
            code,
            message: message.into(),
            data,
        }
    }
}

#[derive(Debug)]
pub enum ErrorCode {
    ParseError,
    InvalidRequest,
    MethodNotFound,
    InvalidParams,
    Internal,
    ServerNotInitialised,
}

impl ErrorCode {
    fn code(&self) -> i32 {
        match self {
            ErrorCode::ParseError => -32700,
            ErrorCode::InvalidRequest => -32600,
            ErrorCode::MethodNotFound => -32601,
            ErrorCode::InvalidParams => -32602,
            ErrorCode::Internal => -32603,
            ErrorCode::ServerNotInitialised => -32002,
        }
    }
}

impl Serialize for ErrorCode {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_i32(self.code())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod request_id_tests {
        use super::*;

        #[test]
        fn deserialises_string_id() {
            let actual: RequestId = serde_json::from_str(r#""id-1""#).unwrap();
            assert_eq!(actual, RequestId::String("id-1".into()))
        }

        #[test]
        fn deserialises_number_id() {
            let actual: RequestId = serde_json::from_str("123").unwrap();
            assert_eq!(actual, RequestId::Number(123))
        }

        #[test]
        fn deserialises_null_id() {
            let actual: RequestId = serde_json::from_str("null").unwrap();
            assert_eq!(actual, RequestId::Null)
        }
    }

    mod message_tests {
        use super::*;

        #[test]
        fn errors_if_not_valid_message() {
            let result: Result<Message, serde_json::Error> = serde_json::from_str("{}");
            assert!(result.is_err())
        }

        #[test]
        fn deserialises_request() {
            let json = r#"{"jsonrpc":"2.0","method":"test","params":true,"id":"123"}"#;
            let actual: Message = serde_json::from_str(json).unwrap();
            assert_eq!(
                actual,
                Message::Request(Request {
                    jsonrpc: Version::V2,
                    method: RequestMethod::Test(true),
                    id: RequestId::String("123".into())
                })
            )
        }

        #[test]
        fn deserialises_batch_request() {
            let json = r#"[{"jsonrpc":"2.0","method":"test","params":true,"id":"123"},{"jsonrpc":"2.0","method":"test","params":false,"id":"456"}]"#;
            let actual: Message = serde_json::from_str(json).unwrap();
            assert_eq!(
                actual,
                Message::BatchRequest(vec![
                    Request {
                        jsonrpc: Version::V2,
                        method: RequestMethod::Test(true),
                        id: RequestId::String("123".into())
                    },
                    Request {
                        jsonrpc: Version::V2,
                        method: RequestMethod::Test(false),
                        id: RequestId::String("456".into())
                    }
                ])
            )
        }

        #[test]
        fn deserialises_notification() {
            let json = r#"{"jsonrpc":"2.0","method":"test","params":true}"#;
            let actual: Message = serde_json::from_str(json).unwrap();
            assert_eq!(
                actual,
                Message::Notification(Notification {
                    jsonrpc: Version::V2,
                    method: NotificationMethod::Test(true),
                })
            )
        }
    }

    mod response_tests {
        use super::*;

        #[test]
        fn serialises_success_response() {
            let response =
                Response::success(&RequestId::String("123".into()), ResponseResult::Null);

            let actual = serde_json::to_string(&response).unwrap();
            assert_eq!(actual, r#"{"jsonrpc":"2.0","result":null,"id":"123"}"#)
        }

        #[test]
        fn serialises_error_response_without_data() {
            let response = Response::error(
                &RequestId::String("123".into()),
                Error::new(ErrorCode::Internal, "Error happened", None),
            );

            let actual = serde_json::to_string(&response).unwrap();
            assert_eq!(
                actual,
                r#"{"jsonrpc":"2.0","error":{"code":-32603,"message":"Error happened","data":null},"id":"123"}"#
            )
        }

        #[test]
        fn serialises_error_response_with_data() {
            let response = Response::error(
                &RequestId::String("123".into()),
                Error::new(
                    ErrorCode::Internal,
                    "Error happened",
                    Some(serde_json::Value::String("some data".into())),
                ),
            );

            let actual = serde_json::to_string(&response).unwrap();
            assert_eq!(
                actual,
                r#"{"jsonrpc":"2.0","error":{"code":-32603,"message":"Error happened","data":"some data"},"id":"123"}"#
            )
        }
    }
}
