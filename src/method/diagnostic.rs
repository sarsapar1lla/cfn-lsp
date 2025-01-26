use crate::model::method::diagnostic::Diagnostic;
use core::str;
use std::{
    fmt::{Debug, Display},
    process::{Command, Output},
};

const CFN_LINT: &str = "cfn-lint";

pub struct LintError {
    message: String,
}

impl Display for LintError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

pub trait Lint: Debug {
    fn lint(&self, uri: &str) -> Result<Vec<Diagnostic>, LintError>;
}

#[derive(Debug, Clone)]
pub struct CfnLinter;

impl Lint for CfnLinter {
    fn lint(&self, uri: &str) -> Result<Vec<Diagnostic>, LintError> {
        let path = extract_file_path(uri);
        tracing::debug!("Invoking cfn-lint for file '{path}'");
        let result = execute_linter(&path)?;

        if result.status.success() {
            Ok(Vec::new())
        } else {
            let response = str::from_utf8(&result.stdout).map_err(|e| LintError {
                message: format!("Linter response is not valid utf-8: {e}"),
            })?;
            let diagnostics: Vec<model::LintDiagnostic> =
                serde_json::from_str(response).map_err(|e| LintError {
                    message: format!("Linter reponse didn't match expected structure: {e}"),
                })?;
            Ok(diagnostics.into_iter().map(From::from).collect())
        }
    }
}

fn execute_linter(uri: &str) -> Result<Output, LintError> {
    Command::new(CFN_LINT)
        .args(["--template", uri, "--format", "json"])
        .output()
        .map_err(|e| LintError {
            message: format!("Failed to invoke '{CFN_LINT}': {e}"),
        })
}

fn extract_file_path(uri: &str) -> String {
    let path = uri.replace("file://", "");
    let path = path.split(":").last().unwrap();
    path.to_string()
}

mod model {
    use crate::model::method::diagnostic;
    use serde::Deserialize;

    #[derive(Debug, Deserialize)]
    #[serde(rename_all = "PascalCase")]
    pub struct LintDiagnostic {
        id: String,
        level: DiagnosticLevel,
        location: Location,
        message: String,
        rule: Rule,
    }

    #[derive(Debug, Deserialize)]
    enum DiagnosticLevel {
        Error,
        Warning,
        Information,
    }

    impl From<DiagnosticLevel> for diagnostic::Severity {
        fn from(value: DiagnosticLevel) -> Self {
            match value {
                DiagnosticLevel::Error => Self::Error,
                DiagnosticLevel::Warning => Self::Warning,
                DiagnosticLevel::Information => Self::Information,
            }
        }
    }

    #[derive(Debug, Deserialize)]
    #[serde(rename_all = "PascalCase")]
    struct Location {
        start: Position,
        end: Position,
    }

    impl From<Location> for diagnostic::Range {
        fn from(value: Location) -> Self {
            Self::new(
                diagnostic::Position::from(value.start),
                diagnostic::Position::from(value.end),
            )
        }
    }

    #[derive(Debug, Deserialize)]
    #[serde(rename_all = "PascalCase")]
    struct Position {
        line_number: usize,
        column_number: usize,
    }

    impl From<Position> for diagnostic::Position {
        fn from(value: Position) -> Self {
            diagnostic::Position::new(value.line_number - 1, value.column_number - 1)
        }
    }

    #[derive(Debug, Deserialize)]
    #[serde(rename_all = "PascalCase")]
    struct Rule {
        id: String,
        source: String,
    }

    impl From<LintDiagnostic> for diagnostic::Diagnostic {
        fn from(value: LintDiagnostic) -> Self {
            Self::builder()
                .range(diagnostic::Range::from(value.location))
                .severity(diagnostic::Severity::from(value.level))
                .code(value.rule.id)
                .code_description(diagnostic::CodeDescription::new(&value.rule.source))
                .source(super::CFN_LINT.into())
                .message(value.message)
                .tags(Vec::new())
                .related_information(Vec::new())
                .build()
        }
    }
}
