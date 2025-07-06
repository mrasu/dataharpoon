mod agent;
mod cli;
mod config;
mod engine;
mod infra;
mod model;
mod repo;
mod server;
mod util;

use crate::cli::cli::run_cli;
use simple_logger::SimpleLogger;
use std::error::Error;

#[tokio::main]
async fn main() {
    SimpleLogger::new()
        .with_level(log::LevelFilter::Info)
        .init()
        .unwrap();

    let res = run_main().await;

    if let Err(e) = res {
        eprintln!("error: {:?}", e)
    };
}

async fn run_main() -> Result<(), Box<dyn Error>> {
    run_cli().await?;

    Ok(())
}
