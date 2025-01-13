use std::{
    fmt::Display,
    io::{Stdout, Write},
};

use crate::model::Response;

const CONTENT_TYPE_HEADER: &[u8] = b"Content-Type: application/vscode-jsonrpc; charset=utf-8\r\n";

pub struct WriteError {
    message: String,
}

impl WriteError {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl Display for WriteError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

pub struct Writer {
    stdout: Stdout,
}

impl Writer {
    pub fn new(stdout: Stdout) -> Self {
        Self { stdout }
    }

    pub fn write(&mut self, response: Response) -> Result<(), WriteError> {
        let json = serde_json::to_string(&response)
            .map_err(|e| WriteError::new(format!("Failed to serialize response: '{e}'")))?;
        self.write_headers(json.len())?;
        self.stdout
            .write(&json.into_bytes())
            .map_err(|e| WriteError::new(format!("Failed to write response to stdout: '{e}'")))?;
        Ok(())
    }

    fn write_headers(&mut self, content_length: usize) -> Result<(), WriteError> {
        self.stdout
            .write(&self.content_length(content_length))
            .map_err(|e| {
                WriteError::new(format!(
                    "Failed to write Content-Length header to stdout: '{e}'"
                ))
            })?;
        self.stdout.write(CONTENT_TYPE_HEADER).map_err(|e| {
            WriteError::new(format!(
                "Failed to write Content-Type header to stdout: '{e}'"
            ))
        })?;
        self.stdout.write(b"\r\n").map_err(|e| {
            WriteError::new(format!(
                "Failed to write end of headers block to stdout: '{e}'"
            ))
        })?;

        Ok(())
    }

    fn content_length(&self, content_length: usize) -> Vec<u8> {
        format!("Content-Length: {}\r\n", content_length).into_bytes()
    }
}
