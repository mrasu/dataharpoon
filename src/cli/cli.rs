use crate::cli::mcp_server::run_mcp_server;
use crate::cli::repl::run_repl;
use crate::config::config::Config;
use clap::{Args, Parser, Subcommand};
use std::error::Error;

const DEFAULT_CONFIG_FILE: &str = "data_harpoon.toml";

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    #[arg(short, long, default_value = DEFAULT_CONFIG_FILE, global = true)]
    config: String,
}

#[derive(Debug, Subcommand)]
pub(super) enum Commands {
    #[command(about = "Start the local server")]
    Serve(ServeArgs),

    #[command(about = "Start the interactive shell")]
    Repl,
}

#[derive(Debug, Args)]
pub(super) struct ServeArgs {
    #[command(subcommand)]
    command: ServeCommands,
}

#[derive(Debug, Subcommand)]
pub(super) enum ServeCommands {
    #[command(about = "Start the MCP server")]
    Mcp,
}

pub async fn run_cli() -> Result<(), Box<dyn Error>> {
    let cli = Cli::parse();

    let config = Config::load_config(cli.config.as_str()).await?;
    match cli.command {
        Some(Commands::Repl) => run_repl(config).await,
        Some(Commands::Serve(args)) => match args.command {
            ServeCommands::Mcp => run_mcp_server(config).await?,
        },
        None => run_repl(config).await,
    }

    Ok(())
}
