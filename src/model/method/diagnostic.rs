use bon::Builder;
use serde::{Deserialize, Serialize};

pub mod publish;
pub mod pull;

#[derive(Debug, Deserialize, Serialize, Builder)]
#[cfg_attr(test, derive(PartialEq, Eq))]
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

#[derive(Debug, Deserialize, Serialize, Clone)]
#[cfg_attr(test, derive(PartialEq, Eq))]
pub struct Position {
    line: usize,
    character: usize,
}

impl Position {
    pub fn new(line: usize, character: usize) -> Self {
        Self { line, character }
    }
}

#[derive(Debug, Deserialize, Serialize)]
#[cfg_attr(test, derive(PartialEq, Eq))]
pub struct Range {
    start: Position,
    end: Position,
}

impl Range {
    pub fn new(start: Position, end: Position) -> Self {
        Self { start, end }
    }
}

#[derive(Debug, Deserialize)]
#[cfg_attr(test, derive(PartialEq, Eq))]
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

#[derive(Debug, Deserialize, Serialize)]
#[cfg_attr(test, derive(PartialEq, Eq))]
pub struct CodeDescription {
    href: String,
}

impl CodeDescription {
    pub fn new(href: &str) -> Self {
        Self { href: href.into() }
    }
}

#[derive(Debug, Deserialize)]
#[cfg_attr(test, derive(PartialEq, Eq))]
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

#[derive(Debug, Deserialize, Serialize)]
#[cfg_attr(test, derive(PartialEq, Eq))]
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

#[derive(Debug, Deserialize, Serialize)]
#[cfg_attr(test, derive(PartialEq, Eq))]
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
