use std::{
    fs::File,
    io::{BufRead, BufReader, Read},
};

pub fn file_reader(path: &str) -> BufReader<File> {
    BufReader::new(File::open(path).unwrap())
}

pub fn message(json: &str) -> Vec<u8> {
    format!("Content-Length: {}\r\n\r\n{}", json.len(), json).into_bytes()
}

pub fn read_message<R>(reader: &mut BufReader<R>) -> String
where
    R: Read,
{
    let mut buffer = String::new();
    reader.read_line(&mut buffer).unwrap();
    let mut buffer = buffer.replace("\r\n", "");
    let content_length: usize = buffer
        .split_whitespace()
        .last()
        .map(|s| s.parse().unwrap())
        .unwrap();

    reader.read_line(&mut buffer).unwrap();
    reader.read_line(&mut buffer).unwrap();

    let mut buffer = Vec::new();
    buffer.resize(content_length, 0);
    reader.read_exact(&mut buffer).unwrap();

    String::from_utf8(buffer).unwrap()
}
