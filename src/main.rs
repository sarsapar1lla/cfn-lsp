#![allow(dead_code)]

use handler::MessageHandler;
use reader::Reader;
use writer::Writer;

mod handler;
mod log;
mod method;
mod model;
mod reader;
mod writer;

fn main() {
    log::init();

    let mut reader = Reader::new(std::io::stdin());
    let mut handler = MessageHandler::default();
    let mut writer = Writer::new(std::io::stdout());

    loop {
        let message = reader.read().unwrap();
        let response = handler.handle(message);
        if let Some(response) = response {
            let result = writer.write(response);
            if let Err(error) = result {
                tracing::error!("{}", error);
            }
        }
    }
}
