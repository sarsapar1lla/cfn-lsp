use std::io::Stdin;

use nom::Parser;

use crate::model::{ErrorCode, Header, Message, RequestId};

#[derive(Debug)]
pub enum ReadError {
    IncompleteMessage,
    MissingContentLength,
    InvalidContentType(String),
    InvalidRequest {
        id: RequestId,
        error_code: ErrorCode,
    },
    Internal(String),
}

pub struct Reader {
    stdin: Stdin,
    buffer: String,
}

impl Reader {
    pub fn new(stdin: Stdin) -> Self {
        Self {
            stdin,
            buffer: String::new(),
        }
    }

    pub fn read(&mut self) -> Result<Message, ReadError> {
        loop {
            self.stdin.read_line(&mut self.buffer).unwrap();
            match self.parse(&self.buffer) {
                Err(ReadError::IncompleteMessage) => {
                    continue;
                }
                result => {
                    self.buffer.clear();
                    return result;
                }
            }
        }
    }

    fn parse(&self, message: &str) -> Result<Message, ReadError> {
        let (remainder, _) =
            Reader::headers(message).map_err(|_| ReadError::MissingContentLength)?;

        let (remainder, _) = Reader::end_of_headers(remainder)
            .map_err(|_: nom::Err<nom::error::Error<&str>>| ReadError::IncompleteMessage)?;

        serde_json::from_str(remainder).map_err(|_| ReadError::InvalidRequest {
            id: RequestId::Null,
            error_code: ErrorCode::InvalidRequest,
        })
    }

    // fn check_charset(headers: &[Header]) -> Result<(), ReadError> {
    //     let charset = headers
    //         .into_iter()
    //         .filter_map(|header| match header {
    //             Header::ContentType {
    //                 content_type: _,
    //                 character_set: charset,
    //             } if !["utf-8", "utf8"].contains(&charset.as_str()) => Some(charset),
    //             _ => None,
    //         })
    //         .last()
    //         .unwrap_or_else(|| String::from(""))

    //     Ok(())
    // }

    fn end_of_headers(message: &str) -> nom::IResult<&str, ()> {
        nom::sequence::terminated(
            nom::character::complete::crlf,
            nom::combinator::not(nom::combinator::eof),
        )
        .map(|_| ())
        .parse(message)
    }

    fn headers(message: &str) -> nom::IResult<&str, Vec<Header>> {
        let parser = nom::branch::permutation((
            Reader::content_length_header,
            nom::combinator::opt(Reader::content_type_header),
        ));
        nom::combinator::map(parser, |(content_length, content_type)| {
            if let Some(content_type) = content_type {
                vec![content_length, content_type]
            } else {
                vec![content_length]
            }
        })
        .parse(message)
    }

    fn content_length_header(message: &str) -> nom::IResult<&str, Header> {
        let parser = nom::sequence::delimited(
            nom::bytes::complete::tag("Content-Length: "),
            nom::character::complete::digit1,
            nom::character::complete::crlf,
        );
        nom::combinator::map_res(parser, |s| u32::from_str_radix(s, 10))
            .map(|content_length| Header::ContentLength(content_length))
            .parse(message)
    }

    fn content_type_header(message: &str) -> nom::IResult<&str, Header> {
        let charset_parser = nom::sequence::preceded(
            nom::bytes::complete::tag("charset="),
            nom::bytes::complete::take_until("\r\n"),
        );
        let content_type_parser = nom::sequence::separated_pair(
            nom::bytes::complete::take_until1(";"),
            nom::bytes::complete::tag("; "),
            charset_parser,
        )
        .map(
            |(content_type, charset): (&str, &str)| Header::ContentType {
                content_type: content_type.into(),
                charset: charset.into(),
            },
        );
        let mut parser = nom::sequence::delimited(
            nom::bytes::complete::tag("Content-Type: "),
            content_type_parser,
            nom::character::complete::crlf,
        );
        parser.parse(message)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod content_length_tests {
        use super::*;

        #[test]
        fn errors_if_not_content_length_header() {
            let result = Reader::content_length_header("invalid");
            assert!(result.is_err())
        }

        #[test]
        fn parses_content_length() {
            let actual = Reader::content_length_header("Content-Length: 123\r\nSomething").unwrap();
            assert_eq!(actual, ("Something", Header::ContentLength(123)))
        }
    }

    mod content_type_tests {
        use super::*;

        #[test]
        fn errors_if_not_content_type_header() {
            let result = Reader::content_type_header("invalid");
            assert!(result.is_err())
        }

        #[test]
        fn parses_content_type() {
            let actual = Reader::content_type_header(
                "Content-Type: application/vscode-jsonrpc; charset=utf-8\r\nSomething",
            )
            .unwrap();
            assert_eq!(
                actual,
                (
                    "Something",
                    Header::ContentType {
                        content_type: "application/vscode-jsonrpc".into(),
                        charset: "utf-8".into()
                    }
                )
            )
        }
    }

    mod headers_tests {
        use super::*;

        #[test]
        fn errors_if_not_headers() {
            let result = Reader::headers("invalid");
            assert!(result.is_err())
        }

        #[test]
        fn errors_if_missing_content_length() {
            let result = Reader::headers("Content-Type: application/json\r\n\r\nSomething");
            assert!(result.is_err())
        }

        #[test]
        fn parses_just_content_length() {
            let actual = Reader::headers("Content-Length: 123\r\n\r\nSomething").unwrap();
            assert_eq!(actual, ("\r\nSomething", vec![Header::ContentLength(123)]))
        }

        #[test]
        fn parses_all_headers() {
            let actual = Reader::headers(
                "Content-Length: 123\r\nContent-Type: application/vscode-jsonrpc; charset=utf-8\r\n\r\nSomething",
            )
            .unwrap();
            assert_eq!(
                actual,
                (
                    "\r\nSomething",
                    vec![
                        Header::ContentLength(123),
                        Header::ContentType {
                            content_type: "application/vscode-jsonrpc".into(),
                            charset: "utf-8".into()
                        }
                    ]
                )
            )
        }

        #[test]
        fn parses_all_headers_in_any_order() {
            let actual = Reader::headers(
                "Content-Type: application/vscode-jsonrpc; charset=utf-8\r\nContent-Length: 123\r\n\r\nSomething",
            )
            .unwrap();
            assert_eq!(
                actual,
                (
                    "\r\nSomething",
                    vec![
                        Header::ContentLength(123),
                        Header::ContentType {
                            content_type: "application/vscode-jsonrpc".into(),
                            charset: "utf-8".into()
                        }
                    ]
                )
            )
        }
    }

    mod end_of_headers_tests {
        use super::*;

        #[test]
        fn errors_if_not_valid_delimiter() {
            let result = Reader::end_of_headers("something");
            assert!(result.is_err())
        }

        #[test]
        fn errors_if_not_message_after_delimiter() {
            let result = Reader::end_of_headers("\r\n");
            assert!(result.is_err())
        }

        #[test]
        fn parses_end_of_headers() {
            let actual = Reader::end_of_headers("\r\nsomething").unwrap();
            assert_eq!(actual, ("something", ()))
        }
    }
}
