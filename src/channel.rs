use std::{
    io::{BufReader, Read, Write},
    net::TcpListener,
};

use crate::cli::Command;

type Input = BufReader<Box<dyn Read>>;
type Output = Box<dyn Write>;

pub fn connect(command: &Command) -> (Input, Output) {
    match command {
        Command::Stdio => stdio(),
        Command::Socket { port } => socket(*port),
    }
}

fn stdio() -> (Input, Output) {
    tracing::info!("Communicating via stdin/out");
    let reader: Box<dyn Read> = Box::new(std::io::stdin());
    let writer = Box::new(std::io::stdout());
    (BufReader::new(reader), writer)
}

fn socket(port: usize) -> (Input, Output) {
    let listener = TcpListener::bind(format!("127.0.0.1:{port}")).expect("Port is available");
    let (stream, address) = listener.accept().expect("Connection accepted");

    tracing::info!("Accepted connection from client at '{address}'");
    let reader: Box<dyn Read> = Box::new(stream.try_clone().expect("Failed to clone TCP stream"));
    let writer = Box::new(stream);
    (BufReader::new(reader), writer)
}
