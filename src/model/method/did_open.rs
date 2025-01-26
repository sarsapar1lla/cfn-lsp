use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
#[cfg_attr(test, derive(PartialEq, Eq))]
pub struct Params {
    #[serde(rename = "textDocument")]
    text_document: TextDocumentItem,
}

impl Params {
    pub fn text_document(&self) -> &TextDocumentItem {
        &self.text_document
    }
}

#[derive(Debug, Deserialize, Serialize)]
#[cfg_attr(test, derive(PartialEq, Eq))]
pub struct TextDocumentItem {
    uri: String,
    #[serde(rename = "languageId")]
    language_id: String,
    version: usize,
    text: String,
}

impl TextDocumentItem {
    pub fn uri(&self) -> &str {
        &self.uri
    }

    pub fn version(&self) -> usize {
        self.version
    }
}
