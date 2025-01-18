use std::{
    io::{BufRead, BufReader, Write},
    net::TcpStream,
    process::Command,
};

mod common;

#[test]
fn lifecycle() {
    let port = "32770"; // TODO: automatically assign available port
    launch_server(port.to_string());

    let mut connection = TcpStream::connect(format!("127.0.0.1:{port}")).unwrap();
    let mut reader = BufReader::new(connection.try_clone().unwrap());

    let mut responses = Vec::new();
    for line in common::file_reader("./tests/resources/full/input.txt").lines() {
        connection
            .write_all(&common::message(&line.unwrap()))
            .unwrap();
        connection.flush().unwrap();
        responses.push(common::read_message(&mut reader));
    }

    // Terminate server
    connection
        .write_all(&common::message(r#"{"jsonrpc":"2.0","method":"exit"}"#))
        .unwrap();
    connection.flush().unwrap();

    let expected: Vec<String> = common::file_reader("./tests/resources/full/output.txt")
        .lines()
        .into_iter()
        .map(Result::unwrap)
        .collect();

    assert_eq!(responses, expected)
}

fn launch_server(port: String) {
    std::thread::spawn(move || {
        Command::new(env!("CARGO_BIN_EXE_cfn-lsp"))
            .args(["socket", "--port", &port])
            .spawn()
            .unwrap();
    });
}
