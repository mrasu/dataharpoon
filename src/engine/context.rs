use crate::config::config::Config;
use crate::config::mcp_server_config::McpServerConfig;
use crate::engine::call_mcp_function::{CALL_MCP_FUNCTION_NAME, CallMcpFunction};
use datafusion::dataframe::DataFrame;
use datafusion::error::Result;
use datafusion::prelude::SessionContext;
use std::collections::HashMap;
use std::sync::Arc;

pub struct Context {
    context: SessionContext,
    context_config: ContextConfig,
}

struct ContextConfig {
    mcp_servers: Arc<HashMap<String, McpServerConfig>>,
}

impl ContextConfig {
    fn new(config: Config) -> ContextConfig {
        let mut mcp_servers = HashMap::<String, McpServerConfig>::new();
        for mcp in config.mcp_servers {
            mcp_servers.insert(mcp.name.clone(), mcp);
        }

        Self {
            mcp_servers: Arc::new(mcp_servers),
        }
    }
}

impl Context {
    pub fn new(config: Config) -> Context {
        let context = SessionContext::new().enable_url_table();
        let context_config = ContextConfig::new(config);

        let call_mcp_func = CallMcpFunction::new(context_config.mcp_servers.clone());
        context.register_udtf(CALL_MCP_FUNCTION_NAME, Arc::new(call_mcp_func));

        Self {
            context,
            context_config,
        }
    }

    pub async fn run_sql(&self, sql: &str) -> Result<DataFrame> {
        self.context.sql(sql).await
    }
}
