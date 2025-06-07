use crate::config::mcp_server_config::McpServerConfig;
use datafusion::common::{Result, exec_err};
use rmcp::model::{CallToolRequestParam, CallToolResult, JsonObject, PaginatedRequestParam};
use rmcp::model::{PaginatedRequestParamInner, Tool};
use rmcp::service::RunningService;
use rmcp::transport::TokioChildProcess;
use rmcp::{RoleClient, ServiceExt};
use std::process::Command as StdCommand;
use std::sync::Arc;
use tokio::process::Command;

pub struct McpClient {
    config: Arc<McpServerConfig>,
}

impl McpClient {
    pub fn new(config: Arc<McpServerConfig>) -> Self {
        Self { config }
    }

    pub async fn list_tools(&self) -> Result<Vec<Tool>> {
        let service = self.connect_service().await?;

        let mut found_tools = Vec::<Tool>::new();
        let mut list_params: PaginatedRequestParam = None;
        loop {
            let response = match service.list_tools(list_params).await {
                Ok(response) => response,
                Err(e) => {
                    return exec_err!(
                        "failed to list tools for mcp-server({}). {}",
                        self.config.name,
                        e
                    );
                }
            };

            response
                .tools
                .iter()
                .for_each(|tool| found_tools.push(tool.clone()));

            let Some(cursor) = response.next_cursor else {
                break;
            };
            list_params = Some(PaginatedRequestParamInner {
                cursor: cursor.into(),
            });
        }

        if let Err(e) = service.cancel().await {
            return exec_err!("failed to stop mcp server({}). {}", self.config.name, e);
        }
        Ok(found_tools)
    }

    pub async fn call_tool(
        &self,
        name: &str,
        arguments: Option<JsonObject>,
    ) -> Result<CallToolResult> {
        let service = self.connect_service().await?;

        let tool_params = CallToolRequestParam {
            name: name.to_string().into(),
            arguments: arguments.clone(),
        };

        let response = match service.call_tool(tool_params).await {
            Ok(response) => response,
            Err(e) => {
                return exec_err!("failed to call mcp-server({}). {:?}", name, e);
            }
        };

        Ok(response)
    }

    async fn connect_service(&self) -> Result<RunningService<RoleClient, ()>> {
        let mut cmd = StdCommand::new(self.config.command.clone());
        for arg in self.config.args.iter() {
            cmd.arg(arg);
        }
        for (k, v) in self.config.env.iter() {
            cmd.env(k, v);
        }

        let tokio_process = TokioChildProcess::new(&mut Command::from(cmd))?;
        Ok(().serve(tokio_process).await?)
    }
}
