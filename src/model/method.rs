use serde::{Deserialize, Serialize};

pub mod diagnostic;
pub mod did_change;
pub mod did_open;
pub mod did_save;
pub mod initialise;
pub mod initialised;

#[derive(Debug, Deserialize, Serialize)]
#[cfg_attr(test, derive(PartialEq, Eq))]
#[serde(tag = "method", content = "params")]
pub enum RequestMethod {
    #[serde(rename = "initialize")]
    Initialise(initialise::Params),

    #[serde(rename = "shutdown")]
    Shutdown,

    #[serde(rename = "textDocument/diagnostic")]
    PullDiagnostics(diagnostic::pull::Params),
}

#[derive(Debug, Deserialize, Serialize)]
#[cfg_attr(test, derive(PartialEq, Eq))]
#[serde(tag = "method", content = "params")]
pub enum NotificationMethod {
    #[serde(rename = "exit")]
    Exit,

    #[serde(rename = "initialized")]
    Initialised(initialised::Params),

    #[serde(rename = "textDocument/didChange")]
    DidChange(did_change::Params),

    #[serde(rename = "textDocument/didClose")]
    DidClose(serde_json::Value),

    #[serde(rename = "textDocument/didOpen")]
    DidOpen(did_open::Params),

    #[serde(rename = "textDocument/didSave")]
    DidSave(did_save::Params),

    #[serde(rename = "textDocument/publishDiagnostics")]
    PublishDiagnostics(diagnostic::publish::Params),
}

#[cfg(test)]
mod tests {
    use super::*;

    mod method_tests {
        use super::*;

        #[test]
        fn errors_if_invalid_method() {
            let result: Result<RequestMethod, serde_json::Error> = serde_json::from_str("invalid");
            assert!(result.is_err())
        }

        #[test]
        fn deserialises_shutdown() {
            let actual: RequestMethod = serde_json::from_str(r#"{"method":"shutdown"}"#).unwrap();
            assert_eq!(actual, RequestMethod::Shutdown)
        }
    }
}
