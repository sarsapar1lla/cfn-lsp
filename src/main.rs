#![allow(dead_code)]

use clap::Parser;
use handler::MessageHandler;
use model::{Message, Response};

mod channel;
mod cli;
mod handler;
mod log;
mod method;
mod model;
mod reader;
mod writer;

fn main() {
    let cli = cli::Cli::parse();
    log::init(cli.debug());

    if let Some(process_id) = cli.client_process_id() {
        tracing::info!("Server spawned by client process {process_id}");
    }
    let (mut input, mut output) = channel::connect(cli.command());
    let mut handler = MessageHandler::new(cli.client_process_id());

    loop {
        let message = reader::read(&mut input);
        let response = match message {
            Ok(message) => handler.handle(message),
            Err(error) => {
                tracing::error!("{error}");
                Some(Message::Response(Response::from(error)))
            }
        };

        if let Some(response) = response {
            if let Err(error) = writer::write(&mut output, &response) {
                tracing::error!("{error}");
            }
        }
    }
}
