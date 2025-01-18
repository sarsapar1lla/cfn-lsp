use std::io::BufRead;
use std::io::BufReader;
use std::io::Read;

use crate::model::Error;
use crate::model::ErrorResponse;
use crate::model::Response;
use crate::model::{ErrorCode, Message, RequestId};

#[derive(Debug)]
pub enum ReadError {
    MalformedHeaders,
    InvalidContentType(String),
    InvalidRequest {
        id: RequestId,
        error_code: ErrorCode,
    },
    Internal(String),
}

impl From<ReadError> for Response {
    fn from(value: ReadError) -> Self {
        match value {
            ReadError::MalformedHeaders => {
                let error = Error::new(ErrorCode::InvalidRequest, "Malformed headers", None);
                Response::Error(ErrorResponse::new(&RequestId::Null, error))
            }
            ReadError::InvalidContentType(content_type) => {
                let error = Error::new(
                    ErrorCode::InvalidRequest,
                    &format!("Invalid content type '{content_type}'"),
                    None,
                );
                Response::Error(ErrorResponse::new(&RequestId::Null, error))
            }
            ReadError::InvalidRequest { id, error_code } => match error_code {
                ErrorCode::ParseError => {
                    let error = Error::new(error_code, "Invalid JSON", None);
                    Response::Error(ErrorResponse::new(&id, error))
                }
                ErrorCode::InvalidRequest => {
                    let error = Error::new(error_code, "Invalid request", None);
                    Response::Error(ErrorResponse::new(&id, error))
                }
                _ => todo!(),
            },
            ReadError::Internal(_) => todo!(),
        }
    }
}

pub fn read<R>(reader: &mut BufReader<R>) -> Result<Message, ReadError>
where
    R: Read,
{
    let mut buffer = String::new();
    loop {
        reader
            .read_line(&mut buffer)
            .map_err(|_| ReadError::Internal("Failed to read from input".into()))?;
        if buffer.ends_with("\r\n\r\n") {
            break;
        }
    }

    let headers = parse::headers(&buffer)?;

    // TODO: check content type and charset

    let mut buffer = buffer.into_bytes();
    buffer.resize(*headers.content_length(), 0);
    reader
        .read_exact(&mut buffer)
        .map_err(|_| ReadError::Internal("Failed to read from input".into()))?;

    if let Ok(message) = serde_json::from_slice(&buffer) {
        Ok(message)
    } else {
        let request_id = request_id(&buffer)?;
        Err(ReadError::InvalidRequest {
            id: request_id,
            error_code: ErrorCode::InvalidRequest,
        })
    }
}

fn request_id(buffer: &[u8]) -> Result<RequestId, ReadError> {
    let value: serde_json::Value =
        serde_json::from_slice(buffer).map_err(|_| ReadError::InvalidRequest {
            id: RequestId::Null,
            error_code: ErrorCode::ParseError,
        })?;
    let request_id = value
        .pointer("id")
        .and_then(|id| serde_json::from_value(id.clone()).ok())
        .unwrap_or(RequestId::Null);
    Ok(request_id)
}

mod parse {
    use nom::{
        branch::permutation,
        bytes::complete::{tag, take_until1},
        character::complete::{crlf, digit1},
        combinator,
        sequence::{delimited, preceded, separated_pair, terminated},
        Parser,
    };

    use crate::model::{ContentType, Headers};

    use super::ReadError;

    pub fn headers(message: &str) -> Result<Headers, ReadError> {
        let parser = combinator::all_consuming(terminated(
            permutation((content_length_header, combinator::opt(content_type_header))),
            crlf,
        ));
        let (_, headers) = combinator::map(parser, |(content_length, content_type)| {
            Headers::new(content_length, content_type.unwrap_or_default())
        })
        .parse(message)
        .map_err(|_| ReadError::MalformedHeaders)?;

        Ok(headers)
    }

    fn content_length_header(message: &str) -> nom::IResult<&str, usize> {
        let parser = delimited(tag("Content-Length: "), digit1, crlf);
        combinator::map_res(parser, str::parse).parse(message)
    }

    fn content_type_header(message: &str) -> nom::IResult<&str, ContentType> {
        let charset_parser = preceded(tag("charset="), take_until1("\r\n"));
        let content_type_parser = separated_pair(take_until1(";"), tag("; "), charset_parser)
            .map(|(content_type, charset): (&str, &str)| ContentType::new(content_type, charset));
        let mut parser = delimited(tag("Content-Type: "), content_type_parser, crlf);
        parser.parse(message)
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        mod content_length_tests {
            use super::*;

            #[test]
            fn errors_if_not_content_length_header() {
                let result = content_length_header("invalid");
                assert!(result.is_err())
            }

            #[test]
            fn parses_content_length() {
                let actual = content_length_header("Content-Length: 123\r\nSomething").unwrap();
                assert_eq!(actual, ("Something", 123))
            }
        }

        mod content_type_tests {
            use super::*;

            #[test]
            fn errors_if_not_content_type_header() {
                let result = content_type_header("invalid");
                assert!(result.is_err())
            }

            #[test]
            fn parses_content_type() {
                let actual = content_type_header(
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
                let result = headers("invalid");
                assert!(result.is_err())
            }

            #[test]
            fn errors_if_missing_content_length() {
                let result = headers("Content-Type: application/json\r\n\r\n");
                assert!(result.is_err())
            }

            #[test]
            fn errors_if_missing_content_remaining() {
                let result = headers("Content-Type: application/json\r\n\r\nSomething");
                assert!(result.is_err())
            }

            #[test]
            fn parses_just_content_length() {
                let actual = headers("Content-Length: 123\r\n\r\n").unwrap();
                assert_eq!(actual, Headers::new(123, ContentType::default()))
            }

            #[test]
            fn parses_all_headers() {
                let actual = headers(
                    "Content-Length: 123\r\nContent-Type: application/json; charset=utf-8\r\n\r\n",
                )
                .unwrap();
                assert_eq!(
                    actual,
                    Headers::new(123, ContentType::new("application/json", "utf-8"))
                )
            }

            #[test]
            fn parses_all_headers_in_any_order() {
                let actual = headers(
                    "Content-Type: application/vscode-jsonrpc; charset=utf8\r\nContent-Length: 123\r\n\r\n",
                )
                .unwrap();
                assert_eq!(
                    actual,
                    Headers::new(123, ContentType::new("application/vscode-jsonrpc", "utf8"))
                )
            }
        }
    }
}
