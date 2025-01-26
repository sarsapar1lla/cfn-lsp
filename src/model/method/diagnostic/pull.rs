use serde::{Deserialize, Serialize};

use super::Diagnostic;

#[derive(Debug, Deserialize, Serialize)]
#[cfg_attr(test, derive(PartialEq, Eq))]
pub struct Params {
    #[serde(rename = "textDocument")]
    text_document: TextDocumentIdentifier,
    identifier: Option<String>,
    #[serde(rename = "previousResultId")]
    previous_result_id: Option<String>,
}

impl Params {
    pub fn uri(&self) -> &str {
        &self.text_document.uri
    }
}

#[derive(Debug, Deserialize, Serialize)]
#[cfg_attr(test, derive(PartialEq, Eq))]
struct TextDocumentIdentifier {
    uri: String,
}
#[derive(Debug, Deserialize, Serialize)]
#[cfg_attr(test, derive(PartialEq, Eq))]
#[serde(untagged)]
pub enum Result {
    Full {
        kind: ReportKind,
        result_id: String,
        items: Vec<Diagnostic>,
    },
    Unchanged {
        kind: ReportKind,
        result_id: String,
    },
}

impl Result {
    pub fn full(result_id: &str, items: Vec<Diagnostic>) -> Self {
        Self::Full {
            kind: ReportKind::Full,
            result_id: result_id.into(),
            items,
        }
    }

    pub fn unchanged(result_id: &str) -> Self {
        Self::Unchanged {
            kind: ReportKind::Unchanged,
            result_id: result_id.into(),
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
#[cfg_attr(test, derive(PartialEq, Eq))]
#[serde(rename_all = "lowercase")]
pub enum ReportKind {
    Full,
    Unchanged,
}
