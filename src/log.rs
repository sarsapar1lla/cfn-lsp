use tracing::Level;
use tracing_appender::rolling;

pub fn init() {
    let file = rolling::minutely("./log", "cfn-lsp");

    tracing_subscriber::fmt()
        .with_writer(file)
        .with_max_level(Level::INFO)
        .init();
}
