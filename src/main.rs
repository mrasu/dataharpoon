mod cli;
mod config;
mod engine;

use crate::cli::repl::run_repl;
use crate::config::config::Config;

#[tokio::main]
async fn main() {
    run_main().await
}

async fn run_main() {
    let config_res = Config::load_config().await;

    let config = match config_res {
        Ok(config) => config,
        Err(e) => {
            return println!("config error: {}", e);
        }
    };

    run_repl(config).await;
}
