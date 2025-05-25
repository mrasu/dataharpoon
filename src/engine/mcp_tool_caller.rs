use crate::config::mcp_server_config::McpServerConfig;
use datafusion::common::exec_err;
use futures::executor::block_on;
use rmcp::model::RawContent::Text;
use rmcp::model::{CallToolRequestParam, JsonObject};
use rmcp::transport::TokioChildProcess;
use rmcp::{ServiceExt, serde_json};
use serde_json::Value;
use std::process::Command as StdCommand;
use tokio::process::Command;

pub(super) struct McpToolCaller {
    tool_name: String,
    arguments: Option<JsonObject>,
}

impl McpToolCaller {
    pub fn new(tool_name: String, arguments: Option<JsonObject>) -> Self {
        Self {
            tool_name,
            arguments,
        }
    }

    pub fn call(&self, config: &McpServerConfig) -> datafusion::common::Result<Vec<Value>> {
        let mut cmd = StdCommand::new(config.command.clone());
        for arg in config.args.iter() {
            cmd.arg(arg);
        }
        for (k, v) in config.env.iter() {
            cmd.env(k, v);
        }

        let tokio_process = TokioChildProcess::new(&mut Command::from(cmd))?;
        let service = block_on(().serve(tokio_process))?;

        let tool_params = CallToolRequestParam {
            name: self.tool_name.clone().into(),
            arguments: self.arguments.clone(),
        };

        let response = match block_on(service.call_tool(tool_params)) {
            Ok(response) => response,
            Err(e) => {
                return exec_err!("failed to call mcp-server({}). {}", self.tool_name, e);
            }
        };

        let response_raw = response.content.first().map(|v| v.clone().raw);
        let Some(Text(response_text)) = response_raw else {
            return exec_err!(
                "mcp-server({}) not return text: {:?}",
                self.tool_name,
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
                    self.tool_name,
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
                    self.tool_name,
                    response_text
                );
            }
        };

        Ok(values)
    }
}
