use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
#[cfg_attr(test, derive(PartialEq, Eq))]
pub struct Params {
    #[serde(rename = "textDocument")]
    text_document: VersionedTextDocumentIdentifier,
}

impl Params {
    pub fn text_document(&self) -> &VersionedTextDocumentIdentifier {
        &self.text_document
    }
}

#[derive(Debug, Deserialize, Serialize)]
#[cfg_attr(test, derive(PartialEq, Eq))]
pub struct VersionedTextDocumentIdentifier {
    version: usize,
    uri: String,
}

impl VersionedTextDocumentIdentifier {
    pub fn version(&self) -> usize {
        self.version
    }

    pub fn uri(&self) -> &str {
        &self.uri
    }
}
