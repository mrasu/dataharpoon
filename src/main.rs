mod cli;
mod config;
mod engine;
mod infra;
mod server;

use crate::cli::cli::run_cli;
use crate::config::config::Config;
use std::error::Error;

#[tokio::main]
async fn main() {
    let res = run_main().await;

    if let Err(e) = res {
        eprint!("error: {:?}", e)
    };
}

async fn run_main() -> Result<(), Box<dyn Error>> {
    let config = Config::load_config().await?;

    run_cli(config).await?;

    Ok(())
}
