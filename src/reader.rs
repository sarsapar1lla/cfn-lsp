use std::io::{Read, Stdin};

use nom::Parser;

use crate::model::{ContentType, ErrorCode, Headers, Message, RequestId};

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
}

impl Reader {
    pub fn new(stdin: Stdin) -> Self {
        Self { stdin }
    }

    pub fn read(&mut self) -> Result<Message, ReadError> {
        let mut buffer = String::new();
        loop {
            self.stdin.read_line(&mut buffer).unwrap();
            if buffer.ends_with("\r\n\r\n") {
                break;
            }
        }

        let (_, headers) = Reader::headers(&buffer).unwrap();

        // TODO: check content type and charset

        let mut buffer = buffer.into_bytes();
        buffer.resize(*headers.content_length(), 0);
        self.stdin.read_exact(&mut buffer).unwrap();

        let content = String::from_utf8(buffer).unwrap();
        Ok(serde_json::from_str(&content).unwrap())
    }

    fn headers(message: &str) -> nom::IResult<&str, Headers> {
        let parser = nom::combinator::all_consuming(nom::sequence::terminated(
            nom::branch::permutation((
                Reader::content_length_header,
                nom::combinator::opt(Reader::content_type_header),
            )),
            nom::character::complete::crlf,
        ));
        nom::combinator::map(parser, |(content_length, content_type)| {
            Headers::new(content_length, content_type.unwrap_or_default())
        })
        .parse(message)
    }

    fn content_length_header(message: &str) -> nom::IResult<&str, usize> {
        let parser = nom::sequence::delimited(
            nom::bytes::complete::tag("Content-Length: "),
            nom::character::complete::digit1,
            nom::character::complete::crlf,
        );
        nom::combinator::map_res(parser, str::parse).parse(message)
    }

    fn content_type_header(message: &str) -> nom::IResult<&str, ContentType> {
        let charset_parser = nom::sequence::preceded(
            nom::bytes::complete::tag("charset="),
            nom::bytes::complete::take_until("\r\n"),
        );
        let content_type_parser = nom::sequence::separated_pair(
            nom::bytes::complete::take_until1(";"),
            nom::bytes::complete::tag("; "),
            charset_parser,
        )
        .map(|(content_type, charset): (&str, &str)| ContentType::new(content_type, charset));
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
            assert_eq!(actual, ("Something", 123))
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
                    ContentType::new("application/vscode-jsonrpc", "utf-8")
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
            let result = Reader::headers("Content-Type: application/json\r\n\r\n");
            assert!(result.is_err())
        }

        #[test]
        fn errors_if_missing_content_remaining() {
            let result = Reader::headers("Content-Type: application/json\r\n\r\nSomething");
            assert!(result.is_err())
        }

        #[test]
        fn parses_just_content_length() {
            let actual = Reader::headers("Content-Length: 123\r\n\r\n").unwrap();
            assert_eq!(actual, ("", Headers::new(123, ContentType::default())))
        }

        #[test]
        fn parses_all_headers() {
            let actual = Reader::headers(
                "Content-Length: 123\r\nContent-Type: application/json; charset=utf-8\r\n\r\n",
            )
            .unwrap();
            assert_eq!(
                actual,
                (
                    "",
                    Headers::new(123, ContentType::new("application/json", "utf-8"))
                )
            )
        }

        #[test]
        fn parses_all_headers_in_any_order() {
            let actual = Reader::headers(
                "Content-Type: application/vscode-jsonrpc; charset=utf8\r\nContent-Length: 123\r\n\r\n",
            )
            .unwrap();
            assert_eq!(
                actual,
                (
                    "",
                    Headers::new(123, ContentType::new("application/vscode-jsonrpc", "utf8"))
                )
            )
        }
    }
}
