use crate::engine::context::Context;
use crate::util::arrow::json::convert_to_json;
use rmcp::model::{
    CallToolResult, Content, Implementation, ProtocolVersion, ServerCapabilities, ServerInfo,
};
use rmcp::tool;
use rmcp::{Error as McpError, ServerHandler};
use std::sync::Arc;

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
        #[schemars(description = "Execute SQL
<long-description>
This MCP server, called DataHarpoon, functions as a database and supports DataFusion SQL, which is largely compatible with standard SQL syntax.
By executing SQL queries, you can retrieve and aggregate data as needed.

It is also possible to read the contents of local files by using the file name, enclosed in single quotes, as the table name.
For example, writing `SELECT * FROM 'awesome.parquet'` returns the contents of `awesome.parquet`.
Both absolute and relative paths are supported for file names.

If you're unsure about the file's data, you can preview a few rows by running a query like `SELECT * FROM 'awesome.parquet' LIMIT 5`.
The supported file formats include Parquet, CSV, Avro, JSON, and others.

**Important:**
Before composing your SQL queries, **you must first execute** the following query:
```sql
SELECT * FROM information_schema.mcp_tools;
```

This query returns a list of available mcp_tools along with their metadata.
Understanding this information is essential to determine the correct arguments to use with the `call_mcp` and `exec_mcp` function.

# Available Tables

## information_schema.mcp_tools

Table Name: information_schema.mcp_tools
Description: Contains metadata for tools that can be invoked using the `call_mcp` and `exec_mcp` function.
Schema: ```sql
  CREATE TABLE information_schema.mcp_tools (
    server_name VARCHAR,     -- Value to be passed as the `server_name` argument of `call_mcp` and `exec_mcp`
    tool_name VARCHAR,       -- Value to be passed as the `tool_name` argument of `call_mcp` and `exec_mcp`
    description VARCHAR,     -- Describes the output or purpose of the tool
    input_schema VARCHAR     -- JSON Schema defining the structure of the `args` parameter for `call_mcp` and `exec_mcp`
  );
```
Example Query: ```sql
  SELECT * FROM information_schema.mcp_tools WHERE server_name = 'awesome_server';
```

# Available Functions

## call_mcp
Function Name: call_mcp
Description: Executes an MCP tool with the given arguments and returns a table generated from the parsed result.
Arguments:
  1. server_name – Name of the MCP server to execute against
  2. tool_name – Name of the MCP tool to be executed
  3. args – Arguments for the MCP tool, formatted according to the input_schema in information_schema.mcp_tools. (Note: Arguments should be specified as a map (e.g., {'key': 'value'}), representing a JSON object defined by the input_schema.)
Examples:
  * When arguments are provided:
    ```sql
    SELECT * FROM call_mcp('awesome_server', 'awesome_tool', {'key': 'value'});
    ```
  * When no arguments are required:
    ```sql
    SELECT * FROM call_mcp('awesome_server', 'awesome_tool');
    ```
## exec_mcp
Function Name: exec_mcp
Description: Executes an MCP tool with the given arguments and returns the response text. exec_mcp accepts the same arguments as call_mcp, but specifically for the value.
Arguments:
  1. server_name – Name of the MCP server to execute against
  2. tool_name – Name of the MCP tool to be executed
  3. args – Arguments for the MCP tool, formatted according to the input_schema in information_schema.mcp_tools. (Note: Arguments should be specified as a map (e.g., {'key': 'value'}), representing a JSON object defined by the input_schema.)
Examples:
  * When arguments are provided:
    ```sql
    SELECT exec_mcp('awesome_server', 'awesome_tool', {'key': 'value'}) AS awesome_value;
    ```
  * When no arguments are required:
    ```sql
    SELECT exec_mcp('awesome_server', 'awesome_tool') AS awesome_value;
    ```

</long-description>")]
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

        let arrow = match df.collect().await {
            Err(e) => {
                return Ok(CallToolResult::error(vec![Content::text(format!(
                    "invalid query. {:?}",
                    e
                ))]));
            }
            Ok(v) => v,
        };

        let result = match convert_to_json(&arrow).await {
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
