use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
#[cfg_attr(test, derive(PartialEq, Eq))]
pub struct Params {
    #[serde(rename = "textDocument")]
    text_document: TextDocumentIdentifier,
}

impl Params {
    pub fn text_document(&self) -> &TextDocumentIdentifier {
        &self.text_document
    }
}

#[derive(Debug, Deserialize, Serialize)]
#[cfg_attr(test, derive(PartialEq, Eq))]
pub struct TextDocumentIdentifier {
    uri: String,
}

impl TextDocumentIdentifier {
    pub fn uri(&self) -> &str {
        &self.uri
    }
}
