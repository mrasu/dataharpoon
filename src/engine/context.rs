use crate::config::config::Config;
use crate::config::mcp_server_config::McpServerConfig;
use crate::engine::information_schema::information_schema_provider::{
    INFORMATION_SCHEMA_NAME, InformationSchemaProvider,
};
use crate::engine::udf::call_mcp_function::{CALL_MCP_FUNCTION_NAME, CallMcpFunction};
use crate::engine::udf::exec_mcp_function::ExecMcpFunction;
use dashmap::DashMap;
use datafusion::dataframe::DataFrame;
use datafusion::error::Result;
use datafusion::logical_expr::ScalarUDF;
use datafusion::prelude::SessionContext;
use std::sync::Arc;

pub struct Context {
    context: SessionContext,
    context_config: ContextConfig,
}

struct ContextConfig {
    mcp_servers: DashMap<String, Arc<McpServerConfig>>,
}

impl ContextConfig {
    fn new(config: Config) -> ContextConfig {
        let mcp_servers = DashMap::<String, Arc<McpServerConfig>>::new();
        for mcp in config.mcp_servers {
            mcp_servers.insert(mcp.name.clone(), Arc::new(mcp));
        }

        Self { mcp_servers }
    }
}

impl Context {
    pub fn new(config: Config) -> Context {
        let context = SessionContext::new().enable_url_table();
        let context_config = ContextConfig::new(config);

        Self::register_information_schema(&context, &context_config);

        let exec_mcp_func =
            ScalarUDF::from(ExecMcpFunction::new(context_config.mcp_servers.clone()));
        context.register_udf(exec_mcp_func);

        let call_mcp_func = CallMcpFunction::new(context_config.mcp_servers.clone());
        context.register_udtf(CALL_MCP_FUNCTION_NAME, Arc::new(call_mcp_func));

        Self {
            context,
            context_config,
        }
    }

    fn register_information_schema(context: &SessionContext, context_config: &ContextConfig) {
        let information_schema_catalog = context
            .copied_config()
            .options()
            .catalog
            .default_catalog
            .clone();

        let information_schema_provider =
            InformationSchemaProvider::new(context_config.mcp_servers.clone());
        context
            .catalog(information_schema_catalog.as_str())
            .unwrap()
            .register_schema(
                INFORMATION_SCHEMA_NAME.into(),
                Arc::new(information_schema_provider),
            )
            .unwrap();
    }

    pub async fn run_sql(&self, sql: &str) -> Result<DataFrame> {
        self.context.sql(sql).await
    }
}
