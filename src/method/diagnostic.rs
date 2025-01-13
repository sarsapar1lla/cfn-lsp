use crate::model::method::diagnostic::{Diagnostic, Params};
use core::str;
use std::{
    fmt::Debug,
    process::{Command, Output},
};

const CFN_LINT: &str = "cfn-lint";

pub trait Lint: Debug {
    fn lint(&self, params: &Params) -> Vec<Diagnostic>;
}

#[derive(Debug, Clone)]
pub struct CfnLinter;

impl CfnLinter {
    fn run_linter(&self, uri: &str) -> Output {
        Command::new(CFN_LINT)
            .arg("--template")
            .arg(uri)
            .output()
            .unwrap()
    }
}

impl Lint for CfnLinter {
    fn lint(&self, params: &Params) -> Vec<Diagnostic> {
        let result = self.run_linter(params.uri());

        if result.status.success() {
            Vec::new()
        } else {
            let response = str::from_utf8(&result.stdout).unwrap();
            parser::parse(response).unwrap()
        }
    }
}

mod parser {

    use nom::{
        bytes::complete::{tag, take_until},
        character::complete::{digit1, line_ending, not_line_ending},
        combinator::{self, all_consuming},
        multi::separated_list1,
        sequence::{delimited, separated_pair, terminated},
        Parser,
    };

    use crate::model::method::diagnostic::{Diagnostic, Position, Range, Severity};

    #[derive(Debug)]
    #[cfg_attr(test, derive(PartialEq, Eq))]
    struct Code {
        value: String,
        severity: Severity,
    }

    pub fn parse(response: &str) -> Result<Vec<Diagnostic>, String> {
        let parser = terminated(separated_list1(line_ending, diagnostic), line_ending);
        let (_, diagnostics) = all_consuming(parser)
            .parse(response)
            .map_err(|_| String::from("placeholder"))?;
        Ok(diagnostics)
    }

    fn diagnostic(response: &str) -> nom::IResult<&str, Diagnostic> {
        let (remainder, code) = code(response)?;
        let (remainder, message) = message(remainder)?;
        let (remainder, range) = range(remainder)?;

        let diagnostic = Diagnostic::builder()
            .range(range)
            .severity(code.severity)
            .code(code.value)
            .message(message)
            .tags(Vec::new())
            .related_information(Vec::new())
            .build();
        Ok((remainder, diagnostic))
    }

    fn range(response: &str) -> nom::IResult<&str, Range> {
        let position_parser = separated_pair(
            combinator::map_res(digit1, str::parse),
            tag(":"),
            combinator::map_res(digit1, str::parse),
        )
        .map(|(line, character)| {
            let position = Position::new(line, character);
            Range::new(position.clone(), position)
        });
        delimited(
            terminated(take_until(":"), tag(":")),
            position_parser,
            line_ending,
        )
        .parse(response)
    }

    fn message(response: &str) -> nom::IResult<&str, String> {
        terminated(not_line_ending, line_ending)
            .map(Into::into)
            .parse(response)
    }

    fn code(response: &str) -> nom::IResult<&str, Code> {
        terminated(take_until(" "), tag(" "))
            .map(|code: &str| match code {
                code if code.starts_with("E") => Code {
                    value: code.into(),
                    severity: Severity::Error,
                },
                code if code.starts_with("W") => Code {
                    value: code.into(),
                    severity: Severity::Warning,
                },
                code if code.starts_with("I") => Code {
                    value: code.into(),
                    severity: Severity::Information,
                },
                // TODO: review
                _ => todo!(),
            })
            .parse(response)
    }

    #[cfg(test)]
    mod parser_tests {
        use super::*;

        mod diagnostic_tests {
            use super::*;

            #[test]
            fn errors_if_not_diagnostic() {
                let result = diagnostic("something");
                assert!(result.is_err())
            }

            #[test]
            fn parses_diagnostic() {
                let actual = diagnostic("E1001 'Resource' is a required property\n/path/to/some/file.yaml:1:1\nsomething").unwrap();
                let expected = Diagnostic::builder()
                    .range(Range::new(Position::new(1, 1), Position::new(1, 1)))
                    .severity(Severity::Error)
                    .code("E1001".into())
                    .message("'Resource' is a required property".into())
                    .tags(Vec::new())
                    .related_information(Vec::new())
                    .build();
                assert_eq!(actual, ("something", expected))
            }

            #[test]
            fn parses_diagnostic_with_carriage_return() {
                let actual = diagnostic("E1001 'Resource' is a required property\r\n/path/to/some/file.yaml:1:1\r\nsomething").unwrap();
                let expected = Diagnostic::builder()
                    .range(Range::new(Position::new(1, 1), Position::new(1, 1)))
                    .severity(Severity::Error)
                    .code("E1001".into())
                    .message("'Resource' is a required property".into())
                    .tags(Vec::new())
                    .related_information(Vec::new())
                    .build();
                assert_eq!(actual, ("something", expected))
            }
        }

        mod range_tests {
            use super::*;

            #[test]
            fn errors_if_not_range() {
                let result = range("something");
                assert!(result.is_err())
            }

            #[test]
            fn parses_range() {
                let actual = range("/path/to/some/file.yaml:103:12\nsomething").unwrap();
                assert_eq!(
                    actual,
                    (
                        "something",
                        Range::new(Position::new(103, 12), Position::new(103, 12))
                    )
                )
            }

            #[test]
            fn parses_range_with_carriage_return() {
                let actual = range("/path/to/some/file.yaml:103:12\r\nsomething").unwrap();
                assert_eq!(
                    actual,
                    (
                        "something",
                        Range::new(Position::new(103, 12), Position::new(103, 12))
                    )
                )
            }
        }

        mod message_tests {
            use super::*;

            #[test]
            fn errors_if_not_message() {
                let result = message("something");
                assert!(result.is_err())
            }

            #[test]
            fn parses_message() {
                let actual = message("'Resources' is required\nsomething").unwrap();
                assert_eq!(actual, ("something", "'Resources' is required".into()))
            }

            #[test]
            fn parses_message_with_carriage_return() {
                let actual = message("'Resources' is required\r\nsomething").unwrap();
                assert_eq!(actual, ("something", "'Resources' is required".into()))
            }
        }

        mod code_tests {
            use super::*;

            // #[test]
            // fn errors_if_not_valid_code() {
            //     let result = code("something ");
            //     assert!(result.is_err())
            // }

            #[test]
            fn parses_error_code() {
                let actual = code("E1234 something").unwrap();
                assert_eq!(
                    actual,
                    (
                        "something",
                        Code {
                            value: "E1234".into(),
                            severity: Severity::Error
                        }
                    )
                )
            }

            #[test]
            fn parses_warning_code() {
                let actual = code("W1234 something").unwrap();
                assert_eq!(
                    actual,
                    (
                        "something",
                        Code {
                            value: "W1234".into(),
                            severity: Severity::Warning
                        }
                    )
                )
            }

            #[test]
            fn parses_information_code() {
                let actual = code("I1234 something").unwrap();
                assert_eq!(
                    actual,
                    (
                        "something",
                        Code {
                            value: "I1234".into(),
                            severity: Severity::Information
                        }
                    )
                )
            }
        }
    }
}
