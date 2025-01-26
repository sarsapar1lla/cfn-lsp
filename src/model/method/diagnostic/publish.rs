use serde::{Deserialize, Serialize};

use super::Diagnostic;

#[derive(Debug, Deserialize, Serialize)]
#[cfg_attr(test, derive(PartialEq, Eq))]
pub struct Params {
    uri: String,
    version: Option<usize>,
    diagnostics: Vec<Diagnostic>,
}

impl Params {
    pub fn new(uri: &str, version: Option<usize>, diagnostics: Vec<Diagnostic>) -> Self {
        Self {
            uri: uri.into(),
            version,
            diagnostics,
        }
    }
}
