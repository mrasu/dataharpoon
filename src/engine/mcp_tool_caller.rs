use crate::config::mcp_server_config::McpServerConfig;
use crate::infra::mcp_client::McpClient;
use datafusion::common::{DataFusionError, exec_err};
use rmcp::model::JsonObject;
use rmcp::model::RawContent::Text;
use rmcp::serde_json;
use serde_json::Value;
use std::sync::{Arc, mpsc};
use std::thread;

pub(super) struct McpToolCaller {
    server_config: Arc<McpServerConfig>,
}

impl McpToolCaller {
    pub fn new(config: Arc<McpServerConfig>) -> Self {
        Self {
            server_config: config,
        }
    }

    pub fn call(
        &self,
        tool_name: String,
        arguments: Option<JsonObject>,
    ) -> datafusion::common::Result<Vec<Value>> {
        let cli = McpClient::new(self.server_config.clone());

        let (tx, rx) = mpsc::channel();

        // Use thread to make it sync.
        thread::spawn(move || {
            let rt = tokio::runtime::Runtime::new().unwrap();
            let result: datafusion::common::Result<_> = rt.block_on(async move {
                let response = cli.call_tool(tool_name.as_str(), arguments).await?;

                return Ok(response);
            });

            tx.send(result).ok();
        });

        let response = rx
            .recv()
            .map_err(|e| DataFusionError::Execution(format!("{:?}", e)))??;

        let response_raw = response.content.first().map(|v| v.clone().raw);
        let Some(Text(response_text)) = response_raw else {
            return exec_err!(
                "mcp-server({}) does not return text: {:?}",
                self.server_config.name,
                response_raw
            );
        };

        self.to_values(response_text.text.as_str())
    }

    fn to_values(&self, response_text: &str) -> datafusion::common::Result<Vec<Value>> {
        let value: Value = match serde_json::from_str(response_text) {
            Ok(val) => val,
            Err(_) => {
                return exec_err!(
                    "mcp-server({}) not return json. response: {}",
                    self.server_config.name,
                    response_text
                );
            }
        };

        // Normalize value to always return rows for the table.
        let values = match value {
            Value::Array(vec) => vec,
            Value::Object(_) => vec![value],
            _ => {
                return exec_err!(
                    "mcp-server({}) not return array or object. {:?}",
                    self.server_config.name,
                    response_text
                );
            }
        };

        Ok(values)
    }
}
