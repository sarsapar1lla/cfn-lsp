use std::fmt::Display;

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

impl Display for ContentType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}; charset={}", self.content_type, self.charset)
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

impl Display for Headers {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Content-Length: {}\r\nContent-Type: {}\r\n\r\n",
            self.content_length, self.content_type
        )
    }
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

impl Display for RequestId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RequestId::String(id) => write!(f, "{id}"),
            RequestId::Number(id) => write!(f, "{id}"),
            RequestId::Null => write!(f, "null"),
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
#[cfg_attr(test, derive(PartialEq, Eq))]
#[serde(untagged)]
pub enum Message {
    Request(Request),
    BatchRequest(Vec<Request>),
    Notification(Notification),
    Response(Response),
}

#[derive(Debug, Deserialize, Serialize)]
#[cfg_attr(test, derive(PartialEq, Eq))]
pub struct Request {
    jsonrpc: Version,
    #[serde(flatten)]
    method: RequestMethod,
    id: RequestId,
}

impl Request {
    pub fn new(id: RequestId, method: RequestMethod) -> Self {
        Self {
            jsonrpc: Version::V2,
            method,
            id,
        }
    }

    pub fn method(&self) -> &RequestMethod {
        &self.method
    }

    pub fn id(&self) -> &RequestId {
        &self.id
    }
}

#[derive(Debug, Deserialize, Serialize)]
#[cfg_attr(test, derive(PartialEq, Eq))]
pub struct Notification {
    jsonrpc: Version,
    #[serde(flatten)]
    method: NotificationMethod,
}

impl Notification {
    pub fn new(method: NotificationMethod) -> Self {
        Self {
            jsonrpc: Version::V2,
            method,
        }
    }

    pub fn method(&self) -> &NotificationMethod {
        &self.method
    }
}

#[derive(Debug, Deserialize, Serialize)]
#[cfg_attr(test, derive(PartialEq, Eq))]
#[serde(untagged)]
pub enum ResponseResult {
    Initialise(initialise::Result),
    PullDiagnostics(diagnostic::pull::Result),
    Null,
}

#[derive(Debug, Deserialize, Serialize)]
#[cfg_attr(test, derive(PartialEq, Eq))]
pub struct SuccessResponse {
    jsonrpc: Version,
    result: ResponseResult,
    id: RequestId,
}

impl SuccessResponse {
    pub fn new(id: &RequestId, result: ResponseResult) -> Self {
        SuccessResponse {
            jsonrpc: Version::V2,
            result,
            id: id.clone(),
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
#[cfg_attr(test, derive(PartialEq, Eq))]
pub struct ErrorResponse {
    jsonrpc: Version,
    error: Error,
    id: RequestId,
}

impl ErrorResponse {
    pub fn new(id: &RequestId, error: Error) -> Self {
        ErrorResponse {
            jsonrpc: Version::V2,
            error,
            id: id.clone(),
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
#[cfg_attr(test, derive(PartialEq, Eq))]
#[serde(untagged)]
pub enum Response {
    Success(SuccessResponse),
    Error(ErrorResponse),
    Batch(Vec<Response>),
}

#[derive(Debug, Deserialize, Serialize)]
#[cfg_attr(test, derive(PartialEq, Eq))]
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

#[derive(Debug, Deserialize)]
#[cfg_attr(test, derive(PartialEq, Eq))]
pub enum ErrorCode {
    ParseError,
    InvalidRequest,
    MethodNotFound,
    InvalidParams,
    Internal,
    ServerNotInitialised,
    ServerAlreadyInitialised,
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
            ErrorCode::ServerAlreadyInitialised => -32003,
        }
    }
}

impl Display for ErrorCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ErrorCode::ParseError => write!(f, "Failed to parse request"),
            ErrorCode::InvalidRequest => write!(f, "Not a valid request"),
            ErrorCode::MethodNotFound => write!(f, "Method not found"),
            ErrorCode::InvalidParams => write!(f, "Invalid method parameters"),
            ErrorCode::Internal => write!(f, "Internal failure"),
            ErrorCode::ServerNotInitialised => write!(f, "Server not initialised"),
            ErrorCode::ServerAlreadyInitialised => write!(f, "Server already initialised"),
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
            let json = r#"{"jsonrpc":"2.0","method":"shutdown","id":"123"}"#;
            let actual: Message = serde_json::from_str(json).unwrap();
            assert_eq!(
                actual,
                Message::Request(Request {
                    jsonrpc: Version::V2,
                    method: RequestMethod::Shutdown,
                    id: RequestId::String("123".into())
                })
            )
        }

        #[test]
        fn deserialises_batch_request() {
            let json = r#"[{"jsonrpc":"2.0","method":"shutdown","id":"123"},{"jsonrpc":"2.0","method":"shutdown","id":"456"}]"#;
            let actual: Message = serde_json::from_str(json).unwrap();
            assert_eq!(
                actual,
                Message::BatchRequest(vec![
                    Request {
                        jsonrpc: Version::V2,
                        method: RequestMethod::Shutdown,
                        id: RequestId::String("123".into())
                    },
                    Request {
                        jsonrpc: Version::V2,
                        method: RequestMethod::Shutdown,
                        id: RequestId::String("456".into())
                    }
                ])
            )
        }

        #[test]
        fn deserialises_notification() {
            let json = r#"{"jsonrpc":"2.0","method":"exit"}"#;
            let actual: Message = serde_json::from_str(json).unwrap();
            assert_eq!(
                actual,
                Message::Notification(Notification {
                    jsonrpc: Version::V2,
                    method: NotificationMethod::Exit,
                })
            )
        }

        #[test]
        fn deserialises_init_request() {
            let json = r#"{"jsonrpc":"2.0","method":"initialize","params":{"capabilities":{"general":{"positionEncodings":["utf-8","utf-32","utf-16"]},"textDocument":{"codeAction":{"codeActionLiteralSupport":{"codeActionKind":{"valueSet":["","quickfix","refactor","refactor.extract","refactor.inline","refactor.rewrite","source","source.organizeImports"]}},"dataSupport":true,"disabledSupport":true,"isPreferredSupport":true,"resolveSupport":{"properties":["edit","command"]}},"completion":{"completionItem":{"deprecatedSupport":true,"insertReplaceSupport":true,"resolveSupport":{"properties":["documentation","detail","additionalTextEdits"]},"snippetSupport":true,"tagSupport":{"valueSet":[1]}},"completionItemKind":{}},"formatting":{"dynamicRegistration":false},"hover":{"contentFormat":["markdown"]},"inlayHint":{"dynamicRegistration":false},"publishDiagnostics":{"tagSupport":{"valueSet":[1,2]},"versionSupport":true},"rename":{"dynamicRegistration":false,"honorsChangeAnnotations":false,"prepareSupport":true},"signatureHelp":{"signatureInformation":{"activeParameterSupport":true,"documentationFormat":["markdown"],"parameterInformation":{"labelOffsetSupport":true}}}},"window":{"workDoneProgress":true},"workspace":{"applyEdit":true,"configuration":true,"didChangeConfiguration":{"dynamicRegistration":false},"didChangeWatchedFiles":{"dynamicRegistration":true,"relativePatternSupport":false},"executeCommand":{"dynamicRegistration":false},"fileOperations":{"didRename":true,"willRename":true},"inlayHint":{"refreshSupport":false},"symbol":{"dynamicRegistration":false},"workspaceEdit":{"documentChanges":true,"failureHandling":"abort","normalizesLineEndings":false,"resourceOperations":["create","rename","delete"]},"workspaceFolders":true}},"clientInfo":{"name":"helix","version":"25.1 (dabfb6ce)"},"processId":12276,"rootPath":"C:\\Users\\Tim\\projects\\cfn-lsp","rootUri":"file:///C:/Users/Tim/projects/cfn-lsp","workspaceFolders":[{"name":"cfn-lsp","uri":"file:///C:/Users/Tim/projects/cfn-lsp"}]},"id":0}"#;
            let result = serde_json::from_str::<Message>(json);
            assert!(result.is_ok())
        }
    }

    mod response_tests {
        use super::*;

        #[test]
        fn serialises_success_response() {
            let success =
                SuccessResponse::new(&RequestId::String("123".into()), ResponseResult::Null);
            let response = Response::Success(success);

            let actual = serde_json::to_string(&response).unwrap();
            assert_eq!(actual, r#"{"jsonrpc":"2.0","result":null,"id":"123"}"#)
        }

        #[test]
        fn serialises_error_response_without_data() {
            let error = ErrorResponse::new(
                &RequestId::String("123".into()),
                Error::new(ErrorCode::Internal, "Error happened", None),
            );
            let response = Response::Error(error);

            let actual = serde_json::to_string(&response).unwrap();
            assert_eq!(
                actual,
                r#"{"jsonrpc":"2.0","error":{"code":-32603,"message":"Error happened","data":null},"id":"123"}"#
            )
        }

        #[test]
        fn serialises_error_response_with_data() {
            let error = ErrorResponse::new(
                &RequestId::String("123".into()),
                Error::new(
                    ErrorCode::Internal,
                    "Error happened",
                    Some(serde_json::Value::String("some data".into())),
                ),
            );
            let response = Response::Error(error);

            let actual = serde_json::to_string(&response).unwrap();
            assert_eq!(
                actual,
                r#"{"jsonrpc":"2.0","error":{"code":-32603,"message":"Error happened","data":"some data"},"id":"123"}"#
            )
        }
    }
}
