use crate::engine::context::Context;
use crate::model::engine::mcp_tool::McpTool;
use datafusion::arrow::array::AsArray;
use datafusion::common::Result;
use std::rc::Rc;

pub struct McpRepo {
    ctx: Rc<Context>,
}

impl McpRepo {
    pub fn new(ctx: Rc<Context>) -> Self {
        Self { ctx }
    }
}

const LIST_MCP_TOOL_SQL: &str = "SELECT server_name, tool_name, description FROM information_schema.mcp_tools ORDER BY server_name, tool_name";

impl McpRepo {
    pub async fn list_mcp_tools(&self) -> Result<Vec<McpTool>> {
        let res = self.ctx.run_sql(LIST_MCP_TOOL_SQL).await?.collect().await?;

        let mut ret = Vec::<McpTool>::new();
        for batch in res {
            let num = batch.num_rows();
            let mut r = vec![
                McpTool {
                    server_name: "".to_string(),
                    tool_name: "".to_string(),
                    description: "".to_string(),
                };
                num
            ];
            let server_name_col = batch.column_by_name("server_name").unwrap();
            for (i, col) in server_name_col.as_string::<i32>().iter().enumerate() {
                r[i].server_name = col.unwrap_or_default().to_string();
            }

            let tool_name_col = batch.column_by_name("tool_name").unwrap();
            for (i, col) in tool_name_col.as_string::<i32>().iter().enumerate() {
                r[i].tool_name = col.unwrap_or_default().to_string();
            }

            let description_col = batch.column_by_name("description").unwrap();
            for (i, col) in description_col.as_string::<i32>().iter().enumerate() {
                r[i].description = col.unwrap_or_default().to_string();
            }

            ret.append(&mut r);
        }

        Ok(ret)
    }
}
