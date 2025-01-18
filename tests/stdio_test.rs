use std::{
    io::{BufRead, BufReader, Write},
    process::{Command, Stdio},
};

mod common;

#[test]
fn lifecycle() {
    let mut command = Command::new(env!("CARGO_BIN_EXE_cfn-lsp"))
        .arg("stdio")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();

    let mut reader = BufReader::new(command.stdout.as_mut().unwrap());
    let writer = command.stdin.as_mut().unwrap();

    let mut responses = Vec::new();
    for line in common::file_reader("./tests/resources/full/input.txt").lines() {
        writer.write_all(&common::message(&line.unwrap())).unwrap();
        writer.flush().unwrap();
        responses.push(common::read_message(&mut reader));
    }

    // Terminate server
    writer
        .write_all(&common::message(r#"{"jsonrpc":"2.0","method":"exit"}"#))
        .unwrap();
    writer.flush().unwrap();
    command.wait().unwrap();

    let expected: Vec<String> = common::file_reader("./tests/resources/full/output.txt")
        .lines()
        .into_iter()
        .map(Result::unwrap)
        .collect();

    assert_eq!(responses, expected)
}
