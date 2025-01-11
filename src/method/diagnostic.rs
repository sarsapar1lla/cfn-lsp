use crate::model::method::diagnostic::{self, Diagnostic, Position, Range};
use std::fmt::Debug;

pub trait Lint: Debug {
    fn lint(&self, params: &diagnostic::Params) -> Vec<Diagnostic>;
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
