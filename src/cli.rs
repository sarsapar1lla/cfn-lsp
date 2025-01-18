use clap::{ArgAction, Parser, Subcommand};

#[derive(Parser)]
#[command(name = env!("CARGO_PKG_NAME"))]
#[command(version = env!("CARGO_PKG_VERSION"))]
#[command(about = "LSP implementation for AWS CloudFormation", long_about = None)]
pub struct Cli {
    /// LSP client process id
    #[arg(long, global = true, visible_alias = "clientProcessId")]
    client_process_id: Option<String>,

    /// Enable debug logging
    #[arg(long, global = true, action = ArgAction::SetTrue)]
    debug: bool,

    #[command(subcommand)]
    command: Command,
}

impl Cli {
    pub fn client_process_id(&self) -> Option<&String> {
        self.client_process_id.as_ref()
    }

    pub fn debug(&self) -> bool {
        self.debug
    }

    pub fn command(&self) -> &Command {
        &self.command
    }
}

#[derive(Subcommand)]
pub enum Command {
    /// Communicate via StdIn/Out
    Stdio,

    /// Communicate via TCP socket
    Socket {
        /// Port to listen on
        #[arg(long)]
        port: usize,
    },
}
