mod cli;
mod config;
mod engine;
mod infra;
mod server;

use crate::cli::cli::run_cli;
use std::error::Error;

#[tokio::main]
async fn main() {
    let res = run_main().await;

    if let Err(e) = res {
        eprintln!("error: {:?}", e)
    };
}

async fn run_main() -> Result<(), Box<dyn Error>> {
    run_cli().await?;

    Ok(())
}
