use crate::config::mcp_server_config::McpServerConfig;
use async_trait::async_trait;
use dashmap::DashMap;
use datafusion::arrow::array::{ArrayRef, ListBuilder, RecordBatch, StringArray, StringBuilder};
use datafusion::arrow::datatypes::{DataType, Field, Schema, SchemaRef};
use datafusion::catalog::{Session, TableProvider};
use datafusion::common::{Result, exec_err, project_schema};
use datafusion::datasource::TableType;
use datafusion::execution::{SendableRecordBatchStream, TaskContext};
use datafusion::logical_expr::Expr;
use datafusion::physical_expr::{EquivalenceProperties, Partitioning};
use datafusion::physical_plan::execution_plan::{Boundedness, EmissionType};
use datafusion::physical_plan::memory::MemoryStream;
use datafusion::physical_plan::{DisplayAs, DisplayFormatType, ExecutionPlan, PlanProperties};
use std::any::Any;
use std::collections::HashMap;
use std::fmt::Formatter;
use std::sync::Arc;

pub(super) const MCP_SERVERS_TABLE_NAME: &str = "mcp_servers";

#[derive(Debug)]
pub(super) struct McpServersTableProvider {
    mcp_servers: DashMap<String, Arc<McpServerConfig>>,
}

impl McpServersTableProvider {
    pub(super) fn new(mcp_servers: DashMap<String, Arc<McpServerConfig>>) -> Self {
        Self { mcp_servers }
    }
}

#[async_trait]
impl TableProvider for McpServersTableProvider {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn schema(&self) -> SchemaRef {
        let schema = Schema::new(vec![
            Arc::new(Field::new("server_name", DataType::Utf8, true)),
            Arc::new(Field::new("command", DataType::Utf8, true)),
            Arc::new(Field::new(
                "args",
                DataType::List(Arc::new(Field::new_list_field(DataType::Utf8, true))),
                true,
            )),
        ]);

        Arc::new(schema)
    }

    fn table_type(&self) -> TableType {
        TableType::View
    }

    async fn scan(
        &self,
        _state: &dyn Session,
        projection: Option<&Vec<usize>>,
        _filters: &[Expr],
        _limit: Option<usize>,
    ) -> datafusion::common::Result<Arc<dyn ExecutionPlan>> {
        Ok(Arc::new(McpServersTableExecutionPlan::new(
            projection,
            self.schema(),
            self.mcp_servers.clone(),
        )))
    }
}

#[derive(Debug)]
struct McpServersTableExecutionPlan {
    plan_properties: PlanProperties,
    mcp_servers: DashMap<String, Arc<McpServerConfig>>,
}

impl McpServersTableExecutionPlan {
    fn new(
        projection: Option<&Vec<usize>>,
        schema: SchemaRef,
        mcp_servers: DashMap<String, Arc<McpServerConfig>>,
    ) -> Self {
        let projected_schema = project_schema(&schema, projection).unwrap();
        let plan_properties = Self::compute_properties(projected_schema);
        Self {
            plan_properties,
            mcp_servers,
        }
    }

    fn compute_properties(schema: SchemaRef) -> PlanProperties {
        let eq_properties = EquivalenceProperties::new(schema);
        PlanProperties::new(
            eq_properties,
            Partitioning::UnknownPartitioning(1),
            EmissionType::Incremental,
            Boundedness::Bounded,
        )
    }
}

impl DisplayAs for McpServersTableExecutionPlan {
    fn fmt_as(&self, _t: DisplayFormatType, f: &mut Formatter) -> std::fmt::Result {
        write!(f, "McpServersTableExecutionPlan")
    }
}

impl ExecutionPlan for McpServersTableExecutionPlan {
    fn name(&self) -> &str {
        "McpServersTableExecutionPlan"
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn properties(&self) -> &PlanProperties {
        &self.plan_properties
    }

    fn children(&self) -> Vec<&Arc<dyn ExecutionPlan>> {
        vec![]
    }

    fn with_new_children(
        self: Arc<Self>,
        _children: Vec<Arc<dyn ExecutionPlan>>,
    ) -> Result<Arc<dyn ExecutionPlan>> {
        Ok(self)
    }

    fn execute(
        &self,
        _partition: usize,
        _context: Arc<TaskContext>,
    ) -> Result<SendableRecordBatchStream> {
        let schema = self.schema();

        let server_columns = self.server_columns();

        let mut columns = Vec::<ArrayRef>::new();
        for field in schema.fields.iter() {
            let Some(column) = server_columns.get(field.name().as_str()) else {
                return exec_err!("unknown column found. column: {}", field.name());
            };
            columns.push(column.clone());
        }

        let Ok(rec) = RecordBatch::try_new(schema.clone(), columns) else {
            return exec_err!("failed to create RecordBatch. schema: {:?}", self.schema());
        };

        let stream = Box::pin(MemoryStream::try_new(vec![rec], self.schema(), None)?);
        Ok(stream)
    }
}

impl McpServersTableExecutionPlan {
    fn server_columns(&self) -> HashMap<&str, ArrayRef> {
        let configs: Vec<Arc<McpServerConfig>> =
            self.mcp_servers.iter().map(|v| v.value().clone()).collect();

        let names: Vec<_> = configs.iter().map(|c| c.name.clone()).collect();
        let names_array = StringArray::from(names);

        let commands: Vec<_> = configs.iter().map(|c| c.command.clone()).collect();
        let commands_array = StringArray::from(commands);

        let args_builder = StringBuilder::new();
        let mut args_list_builder = ListBuilder::new(args_builder);
        for config in configs {
            for arg in config.args.iter() {
                args_list_builder.values().append_value(arg);
            }
            args_list_builder.append(true);
        }

        let result_arrays = HashMap::<&str, ArrayRef>::from([
            ("server_name", Arc::new(names_array) as ArrayRef),
            ("command", Arc::new(commands_array)),
            ("args", Arc::new(args_list_builder.finish())),
        ]);

        result_arrays
    }
}
