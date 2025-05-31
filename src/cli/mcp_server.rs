use crate::config::config::Config;
use crate::engine::context::Context;
use crate::server::mcp::server::McpServer;
use datafusion::common::Result;
use rmcp::service::QuitReason;
use rmcp::{ServiceExt, transport::stdio};
use std::sync::Arc;

pub(super) async fn run_mcp_server(config: Config) -> Result<()> {
    println!("DataHarpoon MCP Server running on stdio");

    let ctx = Context::new(config);
    let service = McpServer::new(Arc::new(ctx)).serve(stdio()).await?;

    match service.waiting().await {
        Ok(QuitReason::Closed) => println!("Closed"),
        Ok(QuitReason::Cancelled) => println!("Cancelled"),
        Err(e) => eprintln!("Error occurred. {:?}", e),
    }

    Ok(())
}
