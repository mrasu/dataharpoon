use crate::cli::mcp_server::run_mcp_server;
use crate::cli::repl::run_repl;
use crate::config::config::Config;
use clap::{Args, Parser, Subcommand};
use datafusion::common::Result;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
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

pub async fn run_cli(config: Config) -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Some(Commands::Repl) => run_repl(config).await,
        Some(Commands::Serve(args)) => match args.command {
            ServeCommands::Mcp => run_mcp_server(config).await?,
        },
        None => run_repl(config).await,
    }

    Ok(())
}
