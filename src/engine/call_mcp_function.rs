use crate::config::mcp_server_config::McpServerConfig;
use crate::engine::mcp_tool_caller::McpToolCaller;
use dashmap::DashMap;
use datafusion::arrow::datatypes::{Schema, SchemaRef};
use datafusion::arrow::json::ReaderBuilder;
use datafusion::arrow::json::reader::infer_json_schema;
use datafusion::catalog::{MemTable, TableFunctionImpl, TableProvider};
use datafusion::common::{Result, exec_err};
use datafusion::common::{ScalarValue, plan_err};
use datafusion::logical_expr::Expr;
use rmcp::model::JsonObject;
use rmcp::serde_json;
use serde_json::Value;
use std::io::Cursor;
use std::sync::Arc;

pub const CALL_MCP_FUNCTION_NAME: &str = "call_mcp";

#[derive(Debug)]
pub struct CallMcpFunction {
    servers: DashMap<String, Arc<McpServerConfig>>,
}

impl CallMcpFunction {
    pub fn new(servers: DashMap<String, Arc<McpServerConfig>>) -> Self {
        Self { servers }
    }
}

impl TableFunctionImpl for CallMcpFunction {
    fn call(&self, args: &[Expr]) -> Result<Arc<dyn TableProvider>> {
        let (server_name, tool_name, tool_arguments) = self.extract_mcp_arguments(args)?;

        let Some(config) = self.servers.get(server_name.as_str()) else {
            return plan_err!(
                "not registered mcp server is used for {}. Server: {}",
                CALL_MCP_FUNCTION_NAME,
                server_name
            );
        };

        let tool_caller = McpToolCaller::new(tool_name.clone(), tool_arguments);
        // TODO: cache result in the same repl session.
        let values = tool_caller.call(config.value())?;

        let schema = self.infer_schema(&values)?;
        self.create_table_provider_for(schema, &values, tool_name)
    }
}

impl CallMcpFunction {
    fn extract_mcp_arguments(&self, args: &[Expr]) -> Result<(String, String, Option<JsonObject>)> {
        let Some(Expr::Literal(ScalarValue::Utf8(Some(server_name)))) = args.get(0) else {
            return plan_err!(
                "first argument for {} must be an string (server-name)",
                CALL_MCP_FUNCTION_NAME
            );
        };
        let Some(Expr::Literal(ScalarValue::Utf8(Some(tool_name)))) = args.get(1) else {
            return plan_err!(
                "second argument for {} must be an string (tool-name)",
                CALL_MCP_FUNCTION_NAME
            );
        };

        let tool_arguments: Option<JsonObject> = self.extract_mcp_tool_arguments(args.get(2))?;

        Ok((server_name.clone(), tool_name.clone(), tool_arguments))
    }

    fn extract_mcp_tool_arguments(&self, expr: Option<&Expr>) -> Result<Option<JsonObject>> {
        let f = match expr {
            None => return Ok(None),
            Some(Expr::ScalarFunction(f)) => f,
            _ => {
                return plan_err!(
                    "Third argument must be an named_struct for {}. e.g. {{'hello': 'world'}}. input: {:?}",
                    CALL_MCP_FUNCTION_NAME,
                    expr
                );
            }
        };

        if f.func.name() != "named_struct" {
            return plan_err!(
                "Third argument must be an named_struct for tool's parameter. e.g. {{'hello': 'world'}}. input: {:?}",
                f
            );
        }
        let mut ret = JsonObject::new();

        let mut current_key: &String = &"".to_string();
        for (i, arg) in f.args.iter().enumerate() {
            if i % 2 == 0 {
                match arg {
                    Expr::Literal(ScalarValue::Utf8(Some(val))) => {
                        current_key = val;
                    }
                    _ => {
                        return plan_err!(
                            "unexpected error. not string for key of named_struct. key: {}",
                            arg
                        );
                    }
                }
            } else {
                match arg {
                    Expr::Literal(ScalarValue::Utf8(Some(val))) => {
                        ret.insert(current_key.to_string(), val.clone().into());
                    }
                    Expr::Literal(ScalarValue::Int64(Some(val))) => {
                        ret.insert(current_key.to_string(), val.clone().into());
                    }

                    Expr::Literal(ScalarValue::Float64(Some(val))) => {
                        ret.insert(current_key.to_string(), val.clone().into());
                    }
                    Expr::Literal(ScalarValue::Boolean(Some(val))) => {
                        ret.insert(current_key.to_string(), val.clone().into());
                    }
                    // TODO: implement other types
                    _ => {
                        return plan_err!(
                            "unimplemented error. only string and int, float, boolean is supported for the value of mcp-tool arguments. value: {}",
                            arg
                        );
                    }
                }
            }
        }

        Ok(Some(ret))
    }

    fn infer_schema(&self, values: &Vec<Value>) -> Result<SchemaRef> {
        let mut schemas = Vec::<Schema>::new();
        for val in values.iter().take(10) {
            // TODO: create a way to infer values as is. dumping to json might be slow.
            let text = serde_json::to_string(val).unwrap();
            let cursor = Cursor::new(text.clone());
            let (schema, _) = infer_json_schema(cursor, Some(100))?;

            schemas.push(schema)
        }
        let schema = Schema::try_merge(schemas)?;

        Ok(Arc::new(schema))
    }

    fn create_table_provider_for(
        &self,
        schema: SchemaRef,
        values: &Vec<Value>,
        tool_name: String,
    ) -> Result<Arc<dyn TableProvider>> {
        let mut decoder = ReaderBuilder::new(schema.clone()).build_decoder()?;
        decoder.serialize(&values)?;
        let batch = match decoder.flush()? {
            Some(batch) => batch,
            None => {
                return exec_err!("failed to deserialize mcp-server({}) response", tool_name);
            }
        };

        let mem = MemTable::try_new(schema, vec![vec![batch]])?;
        Ok(Arc::new(mem))
    }
}
