use crate::config::config::Config;
use crate::config::mcp_server_config::McpServerConfig;
use datafusion::dataframe::DataFrame;
use datafusion::error::Result;
use datafusion::prelude::SessionContext;
use std::collections::HashMap;

pub struct Context {
    context: SessionContext,
    context_config: ContextConfig,
}

struct ContextConfig {
    mcp_servers: HashMap<String, McpServerConfig>,
}

impl ContextConfig {
    fn new(config: Config) -> ContextConfig {
        let mut mcp_servers = HashMap::<String, McpServerConfig>::new();
        for mcp in config.mcp_servers {
            mcp_servers.insert(mcp.name.clone(), mcp);
        }

        Self { mcp_servers }
    }
}

impl Context {
    pub fn new(config: Config) -> Context {
        let context = SessionContext::new().enable_url_table();
        let context_config = ContextConfig::new(config);

        Self {
            context,
            context_config,
        }
    }

    pub async fn run_sql(&self, sql: &str) -> Result<DataFrame> {
        self.context.sql(sql).await
    }
}
