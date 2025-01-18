use tracing::Level;
use tracing_appender::rolling;

pub fn init(debug: bool) {
    let file = rolling::hourly("./log", env!("CARGO_PKG_NAME"));
    let log_level = if debug { Level::DEBUG } else { Level::INFO };

    tracing_subscriber::fmt()
        .with_writer(file)
        .with_max_level(log_level)
        .init();
}
