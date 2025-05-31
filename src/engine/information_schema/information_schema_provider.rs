use crate::config::mcp_server_config::McpServerConfig;
use crate::engine::information_schema::mcp_servers_table_provider::{
    MCP_SERVERS_TABLE_NAME, McpServersTableProvider,
};
use crate::engine::information_schema::mcp_tools_table_provider::{
    MCP_TOOLS_TABLE_NAME, McpToolsTableProvider,
};
use async_trait::async_trait;
use dashmap::DashMap;
use datafusion::catalog::{SchemaProvider, TableProvider};
use datafusion::common::DataFusionError;
use std::any::Any;
use std::sync::Arc;

pub const INFORMATION_SCHEMA_NAME: &str = "information_schema";

#[derive(Debug)]
pub struct InformationSchemaProvider {
    tables: DashMap<String, Arc<dyn TableProvider>>,
}

impl InformationSchemaProvider {
    pub fn new(mcp_servers: DashMap<String, Arc<McpServerConfig>>) -> Self {
        Self {
            tables: Self::initialize_tables(mcp_servers),
        }
    }

    fn initialize_tables(
        mcp_servers: DashMap<String, Arc<McpServerConfig>>,
    ) -> DashMap<String, Arc<dyn TableProvider>> {
        let tables = DashMap::<String, Arc<dyn TableProvider>>::new();
        tables.insert(
            MCP_TOOLS_TABLE_NAME.to_string(),
            Arc::new(McpToolsTableProvider::new(mcp_servers.clone())),
        );
        tables.insert(
            MCP_SERVERS_TABLE_NAME.to_string(),
            Arc::new(McpServersTableProvider::new(mcp_servers)),
        );

        tables
    }
}

#[async_trait]
impl SchemaProvider for InformationSchemaProvider {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn table_names(&self) -> Vec<String> {
        self.tables.iter().map(|t| t.key().clone()).collect()
    }

    async fn table(&self, name: &str) -> Result<Option<Arc<dyn TableProvider>>, DataFusionError> {
        let table = self.tables.get(name).map(|t| t.value().clone());
        Ok(table)
    }

    fn table_exist(&self, name: &str) -> bool {
        self.tables.contains_key(name)
    }
}
