use serde::Deserialize;

pub mod diagnostic;
pub mod initialise;

#[derive(Debug, Deserialize)]
#[cfg_attr(test, derive(PartialEq, Eq))]
#[serde(tag = "method", content = "params")]
pub enum RequestMethod {
    #[serde(rename = "initialize")]
    Initialise(initialise::Params),
    #[serde(rename = "shutdown")]
    Shutdown,
    #[serde(rename = "textDocument/diagnostic")]
    TextDocumentDiagnostic(diagnostic::Params),
}

#[derive(Debug, Deserialize)]
#[cfg_attr(test, derive(PartialEq, Eq))]
#[serde(tag = "method", content = "params")]
pub enum NotificationMethod {
    #[serde(rename = "exit")]
    Exit,
    #[serde(rename = "initialized")]
    Initialised,
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
