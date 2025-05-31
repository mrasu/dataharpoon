use crate::engine::context::Context;
use datafusion::arrow::json::ArrayWriter;
use datafusion::prelude::DataFrame;
use rmcp::model::{
    CallToolResult, Content, Implementation, ProtocolVersion, ServerCapabilities, ServerInfo,
};
use rmcp::tool;
use rmcp::{Error as McpError, ServerHandler};
use std::sync::Arc;
/*
{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{"roots":{"listChanged":true},"sampling":{}},"clientInfo":{"name":"ExampleClient","version":"1.0.0"}}}
{"jsonrpc":"2.0","method":"notifications/initialized"}
{"jsonrpc":"2.0","id":2,"method":"tools/call","params":{"name":"run_query","arguments":{"query":"select server_name, tool_name, description from information_schema.mcp_tools where server_name = 'time';"}}}
 */
#[derive(Clone)]
pub struct McpServer {
    context: Arc<Context>,
}

#[tool(tool_box)]
impl McpServer {
    pub fn new(context: Arc<Context>) -> Self {
        Self { context }
    }

    #[tool(description = "Run SQL query")]
    async fn run_query(
        &self,
        #[tool(param)]
        #[schemars(description = "SQL to be run")]
        query: String,
    ) -> Result<CallToolResult, McpError> {
        let res = self.context.run_sql(query.as_str()).await;
        let df = match res {
            Ok(df) => df,
            Err(e) => {
                return Ok(CallToolResult::error(vec![Content::text(format!(
                    "invalid query. {:?}",
                    e
                ))]));
            }
        };

        let result = match self.convert_to_json(df).await {
            Ok(result) => result,
            Err(e) => {
                return Ok(CallToolResult::error(vec![Content::text(format!(
                    "invalid query. {:?}",
                    e
                ))]));
            }
        };

        let result_json = String::from_utf8_lossy(result.as_slice());
        Ok(CallToolResult::success(vec![Content::text(result_json)]))
    }

    async fn convert_to_json(&self, df: DataFrame) -> datafusion::common::Result<Vec<u8>> {
        let result = df.collect().await?;
        let mut buf = Vec::with_capacity(1024);
        let mut writer = ArrayWriter::new(&mut buf);
        for batch in result {
            writer.write(&batch)?;
        }
        writer.finish()?;

        Ok(buf)
    }
}

#[tool(tool_box)]
impl ServerHandler for McpServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            protocol_version: ProtocolVersion::default(),
            capabilities: ServerCapabilities::builder().enable_tools().build(),
            server_info: Implementation {
                name: "DataHarpoon".into(),
                version: "v0.0.1".into(),
            },
            instructions: None,
        }
    }
}
