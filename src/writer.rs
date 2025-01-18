use std::{fmt::Display, io::Write};

use crate::model::{ContentType, Headers, Response};

pub struct WriteError(String);

impl Display for WriteError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

pub fn write<W>(writer: &mut W, response: &Response) -> Result<(), WriteError>
where
    W: Write,
{
    let json = serde_json::to_string(&response)
        .map_err(|e| WriteError(format!("Failed to serialize response: '{e}'")))?;
    let headers = Headers::new(json.len(), ContentType::default());
    let message = format!("{headers}{json}");
    writer
        .write_all(&message.into_bytes())
        .map_err(|e| WriteError(format!("Failed to write response to stdout: '{e}'")))?;

    writer
        .flush()
        .map_err(|e| WriteError(format!("Failed to flush written bytes: '{e}'")))
}
