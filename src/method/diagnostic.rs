use crate::model::method::diagnostic::{self, Diagnostic, Position, Range};
use std::{fmt::Debug, process::Command};

const CFN_LINT: &str = "cfn-lint";

pub trait Lint: Debug {
    fn lint(&self, params: &diagnostic::Params) -> Vec<Diagnostic>;
}

#[derive(Debug, Clone)]
pub struct CfnLinter;

impl Lint for CfnLinter {
    fn lint(&self, params: &diagnostic::Params) -> Vec<Diagnostic> {
        let result = Command::new(CFN_LINT)
            .arg("--template")
            .arg(params.uri())
            .output()
            .unwrap();

        tracing::info!("{}", String::from_utf8(result.stdout).unwrap());

        if result.status.success() {
            Vec::new()
        } else {
            vec![Diagnostic::builder()
                .range(Range::new(Position::new(0, 0), Position::new(0, 5)))
                .severity(diagnostic::Severity::Error)
                .code("E123".into())
                .message("Something's busted".into())
                .tags(Vec::new())
                .related_information(Vec::new())
                .build()]
        }
    }
}

#[derive(Debug, Clone)]
pub struct MockLinter;

impl Lint for MockLinter {
    fn lint(&self, _: &diagnostic::Params) -> Vec<Diagnostic> {
        vec![Diagnostic::builder()
            .range(Range::new(Position::new(0, 0), Position::new(0, 5)))
            .severity(diagnostic::Severity::Error)
            .code("E123".into())
            .message("Something's busted".into())
            .tags(Vec::new())
            .related_information(Vec::new())
            .build()]
    }
}
