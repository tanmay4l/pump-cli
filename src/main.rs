mod cmd;
mod config;
mod core;
mod output;
mod rpc;
mod wallet;

use clap::Parser;
use cmd::Cli;

#[tokio::main]
async fn main() {
    let cli = Cli::parse();
    if let Err(e) = cmd::run(cli).await {
        output::error(&e.to_string());
        std::process::exit(1);
    }
}
