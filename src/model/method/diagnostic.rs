use bon::Builder;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
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

#[derive(Debug, Deserialize)]
#[cfg_attr(test, derive(PartialEq, Eq))]
struct TextDocumentIdentifier {
    uri: String,
}

#[derive(Debug, Serialize)]
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

#[derive(Debug, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum ReportKind {
    Full,
    Unchanged,
}

#[derive(Debug, Serialize, Builder)]
#[serde(rename_all = "camelCase")]
pub struct Diagnostic {
    range: Range,
    severity: Severity,
    code: String,
    code_description: Option<CodeDescription>,
    source: Option<String>,
    message: String,
    tags: Vec<Tag>,
    related_information: Vec<RelatedInformation>,
    data: Option<serde_json::Value>,
}

#[derive(Debug, Serialize)]
pub struct Position {
    line: u32,
    character: u32,
}

impl Position {
    pub fn new(line: u32, character: u32) -> Self {
        Self { line, character }
    }
}

#[derive(Debug, Serialize)]
pub struct Range {
    start: Position,
    end: Position,
}

impl Range {
    pub fn new(start: Position, end: Position) -> Self {
        Self { start, end }
    }
}

#[derive(Debug)]
pub enum Severity {
    Error,
    Warning,
    Information,
    Hint,
}

impl Severity {
    fn value(&self) -> u8 {
        match self {
            Self::Error => 1,
            Self::Warning => 2,
            Self::Information => 3,
            Self::Hint => 4,
        }
    }
}

impl Serialize for Severity {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_u8(self.value())
    }
}

#[derive(Debug, Serialize)]
pub struct CodeDescription {
    href: String,
}

impl CodeDescription {
    pub fn new(href: &str) -> Self {
        Self { href: href.into() }
    }
}

#[derive(Debug)]
pub enum Tag {
    Unnecessary,
    Deprecated,
}

impl Tag {
    fn value(&self) -> u8 {
        match self {
            Self::Unnecessary => 1,
            Self::Deprecated => 2,
        }
    }
}

impl Serialize for Tag {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_u8(self.value())
    }
}

#[derive(Debug, Serialize)]
pub struct RelatedInformation {
    location: Location,
    message: String,
}

impl RelatedInformation {
    pub fn new(location: Location, message: &str) -> Self {
        Self {
            location,
            message: message.into(),
        }
    }
}

#[derive(Debug, Serialize)]
pub struct Location {
    uri: String,
    range: Range,
}

impl Location {
    pub fn new(uri: &str, range: Range) -> Self {
        Self {
            uri: uri.into(),
            range,
        }
    }
}
