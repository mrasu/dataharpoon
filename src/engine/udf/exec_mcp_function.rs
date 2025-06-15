use crate::config::mcp_server_config::McpServerConfig;
use crate::engine::mcp_tool_caller::McpToolCaller;
use crate::infra::mcp_client::McpClient;
use dashmap::DashMap;
use datafusion::arrow::array::{
    Array, AsArray, BooleanArray, Float64Array, Int64Array, StringArray, StructArray,
};
use datafusion::arrow::datatypes::DataType;
use datafusion::common::Result;
use datafusion::common::plan_err;
use datafusion::logical_expr::{
    ColumnarValue, ScalarFunctionArgs, ScalarUDFImpl, Signature, TypeSignature, Volatility,
};
use rmcp::model::JsonObject;
use serde_json::{Map, Value};
use std::any::Any;
use std::sync::Arc;
use std::vec::Vec as StdVec;

const EXEC_MCP_FUNCTION_NAME: &str = "exec_mcp";

#[derive(Debug)]
pub struct ExecMcpFunction {
    servers: DashMap<String, Arc<McpServerConfig>>,
    signature: Signature,
}

impl ExecMcpFunction {
    pub fn new(servers: DashMap<String, Arc<McpServerConfig>>) -> Self {
        Self {
            signature: Signature::one_of(
                vec![
                    TypeSignature::Uniform(2, vec![DataType::Utf8]),
                    // for vec![Utf8, Utf8, Struct]
                    TypeSignature::Any(3),
                ],
                Volatility::Stable,
            ),
            servers,
        }
    }
}

impl ScalarUDFImpl for ExecMcpFunction {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn name(&self) -> &str {
        EXEC_MCP_FUNCTION_NAME
    }

    fn signature(&self) -> &Signature {
        &self.signature
    }

    fn return_type(&self, _arg_types: &[DataType]) -> Result<DataType> {
        Ok(DataType::Utf8)
    }

    fn invoke_with_args(&self, args: ScalarFunctionArgs) -> Result<ColumnarValue> {
        let mcp_args = self.validate_args(&args)?;

        let mut responses = Vec::<String>::new();
        for (server_name, tool_name, tool_arguments) in mcp_args {
            let Some(config) = self.servers.get(server_name.as_str()) else {
                return plan_err!(
                    "not registered mcp server is used for {}. Server: {}",
                    EXEC_MCP_FUNCTION_NAME,
                    server_name
                );
            };

            let tool_caller = McpToolCaller::new(config.clone());
            let response = tool_caller.call_as_string(tool_name.clone(), tool_arguments)?;
            responses.push(response);
        }

        Ok(ColumnarValue::Array(Arc::new(StringArray::from(responses))))
    }
}

impl ExecMcpFunction {
    fn validate_args(
        &self,
        args: &ScalarFunctionArgs,
    ) -> Result<Vec<(String, String, Option<JsonObject>)>> {
        let arguments = &args.args;
        self.check_arg_types(arguments)?;

        let args = ColumnarValue::values_to_arrays(arguments)?;

        let server_names = args[0].as_string::<i32>();
        let tool_names = args[1].as_string::<i32>();
        let tool_args = match args.get(2) {
            Some(args) => Some(self.arrow_struct_array_to_json_object(args.as_struct())?),
            None => None,
        };

        let mut results = StdVec::<(String, String, Option<JsonObject>)>::new();

        let mut server_name_iter = server_names.iter();
        let mut tool_name_iter = tool_names.iter();
        for i in 0..server_names.len() {
            let server_name = server_name_iter.next().flatten().unwrap_or("");
            let tool_name = tool_name_iter.next().flatten().unwrap_or("");

            match &tool_args {
                Some(tool_args) => results.push((
                    server_name.to_string(),
                    tool_name.to_string(),
                    tool_args.get(i).map(|v| v.clone()),
                )),
                None => results.push((server_name.to_string(), tool_name.to_string(), None)),
            }
        }

        Ok(results)
    }

    fn check_arg_types(&self, arguments: &Vec<ColumnarValue>) -> Result<()> {
        let first_arg = match arguments.get(0) {
            Some(v) => v,
            None => {
                return plan_err!(
                    "the first argument not exists for {}",
                    EXEC_MCP_FUNCTION_NAME
                );
            }
        };

        if first_arg.data_type() != DataType::Utf8 {
            return plan_err!(
                "the first argument({}) must be UTF8(String) for {}",
                first_arg.data_type(),
                EXEC_MCP_FUNCTION_NAME
            );
        }

        let second_arg = match arguments.get(1) {
            Some(v) => v,
            None => {
                return plan_err!(
                    "the second argument not exists for {}",
                    EXEC_MCP_FUNCTION_NAME
                );
            }
        };

        if second_arg.data_type() != DataType::Utf8 {
            return plan_err!(
                "the second argument({}) must be UTF8(String) for {}",
                second_arg.data_type(),
                EXEC_MCP_FUNCTION_NAME
            );
        }

        if arguments.len() < 3 {
            return Ok(());
        }

        let third_arg = arguments
            .get(2)
            .expect("no third arguments  but len() is > 2");

        if !matches!(third_arg.data_type(), DataType::Struct(_)) {
            return plan_err!(
                "the third argument({}) must be Struct for {}",
                third_arg.data_type(),
                EXEC_MCP_FUNCTION_NAME
            );
        }

        Ok(())
    }

    fn arrow_struct_array_to_json_object(&self, array: &StructArray) -> Result<Vec<JsonObject>> {
        let num_rows = array.len();
        let fields: Vec<_> = array.fields().iter().map(|f| f.name()).collect();

        let mut result = Vec::with_capacity(num_rows);

        for row in 0..num_rows {
            let mut map = Map::new();

            for (i, col) in array.columns().iter().enumerate() {
                let field_name = fields[i].clone();
                let value = self.array_value_to_json(col, row)?;
                map.insert(field_name, value);
            }

            result.push(map);
        }

        Ok(result)
    }

    fn array_value_to_json(&self, array: &dyn Array, index: usize) -> Result<Value> {
        if array.is_null(index) {
            return Ok(Value::Null);
        }

        match array.data_type() {
            DataType::Utf8 => {
                let arr = array.as_any().downcast_ref::<StringArray>().unwrap();
                Ok(Value::String(arr.value(index).to_string()))
            }
            DataType::Int64 => {
                let arr = array.as_any().downcast_ref::<Int64Array>().unwrap();
                Ok(Value::Number(arr.value(index).into()))
            }
            DataType::Float64 => {
                let arr = array.as_any().downcast_ref::<Float64Array>().unwrap();
                let ret = serde_json::Number::from_f64(arr.value(index))
                    .unwrap_or(Value::Null.as_number().unwrap().clone());
                Ok(Value::Number(ret))
            }
            DataType::Boolean => {
                let arr = array.as_any().downcast_ref::<BooleanArray>().unwrap();
                Ok(Value::Bool(arr.value(index)))
            }
            _ => {
                plan_err!(
                    "unimplemented error. only string and int, float, boolean is supported for the value of mcp-tool arguments. value: {}",
                    array.data_type()
                )
            }
        }
    }
}
