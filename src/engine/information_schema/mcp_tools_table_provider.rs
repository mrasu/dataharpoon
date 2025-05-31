use crate::config::mcp_server_config::McpServerConfig;
use crate::infra::mcp_client::McpClient;
use async_trait::async_trait;
use dashmap::DashMap;
use datafusion::arrow::array::{ArrayRef, RecordBatch, StringArray};
use datafusion::arrow::datatypes::{DataType, Field, Schema, SchemaRef};
use datafusion::catalog::{Session, TableProvider};
use datafusion::common::{Result, ScalarValue, exec_err, project_schema};
use datafusion::datasource::TableType;
use datafusion::execution::{RecordBatchStream, SendableRecordBatchStream, TaskContext};
use datafusion::logical_expr::Expr;
use datafusion::logical_expr::Operator;
use datafusion::logical_expr::TableProviderFilterPushDown;
use datafusion::physical_expr::{EquivalenceProperties, Partitioning};
use datafusion::physical_plan::execution_plan::{Boundedness, EmissionType};
use datafusion::physical_plan::{DisplayAs, DisplayFormatType, ExecutionPlan, PlanProperties};
use futures::Stream;
use rmcp::model::Tool;
use std::any::Any;
use std::collections::HashMap;
use std::fmt::Formatter;
use std::pin::Pin;
use std::sync::Arc;
use std::task::Poll::Ready;
use std::task::{Context, Poll};

pub(super) const MCP_TOOLS_TABLE_NAME: &str = "mcp_tools";

#[derive(Debug)]
pub(super) struct McpToolsTableProvider {
    mcp_servers: DashMap<String, Arc<McpServerConfig>>,
}

impl McpToolsTableProvider {
    pub(super) fn new(mcp_servers: DashMap<String, Arc<McpServerConfig>>) -> Self {
        Self { mcp_servers }
    }
}

#[async_trait]
impl TableProvider for McpToolsTableProvider {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn schema(&self) -> SchemaRef {
        let schema = Schema::new(vec![
            Arc::new(Field::new("server_name", DataType::Utf8, true)),
            Arc::new(Field::new("tool_name", DataType::Utf8, true)),
            Arc::new(Field::new("description", DataType::Utf8, true)),
            Arc::new(Field::new("input_schema", DataType::Utf8, true)),
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
        filters: &[Expr],
        _limit: Option<usize>,
    ) -> Result<Arc<dyn ExecutionPlan>> {
        let f: Vec<&Expr> = filters.iter().map(|x| x).collect();
        Ok(Arc::new(McpToolsTableExecutionPlan::new(
            self.mcp_servers.clone(),
            projection,
            &f,
            self.schema(),
        )))
    }

    fn supports_filters_pushdown(
        &self,
        filters: &[&Expr],
    ) -> Result<Vec<TableProviderFilterPushDown>> {
        let support = server_name_filters(filters)
            .iter()
            .map(|x| match x {
                Some(_) => TableProviderFilterPushDown::Exact,
                None => TableProviderFilterPushDown::Unsupported,
            })
            .collect();

        Ok(support)
    }
}

#[derive(Debug)]
struct McpToolsTableExecutionPlan {
    server_name_filters: Option<Vec<String>>,
    mcp_servers: DashMap<String, Arc<McpServerConfig>>,
    plan_properties: PlanProperties,
}

fn server_name_filters(filters: &[&Expr]) -> Vec<Option<String>> {
    let server_name_filter_vec: Vec<Option<String>> = filters
        .iter()
        .map(|expr| match expr {
            Expr::BinaryExpr(bin_expr) => {
                if bin_expr.op != Operator::Eq {
                    return None;
                }
                let Expr::Literal(ScalarValue::Utf8(Some(val))) = *bin_expr.right.clone() else {
                    return None;
                };

                bin_expr
                    .left
                    .try_as_col()
                    .map(|column| {
                        if column.name == "server_name" {
                            return Some(val);
                        }
                        None
                    })
                    .unwrap_or(None)
            }
            _ => None,
        })
        .collect();

    server_name_filter_vec
}

impl McpToolsTableExecutionPlan {
    fn new(
        mcp_servers: DashMap<String, Arc<McpServerConfig>>,
        projection: Option<&Vec<usize>>,
        filters: &[&Expr],
        schema: SchemaRef,
    ) -> Self {
        let server_name_filter_vec: Vec<_> = server_name_filters(filters)
            .iter()
            .filter_map(|x| x.clone())
            .collect();
        let server_name_filters = if server_name_filter_vec.len() > 0 {
            Some(server_name_filter_vec)
        } else {
            None
        };

        let projected_schema = project_schema(&schema, projection).unwrap();
        let plan_properties = Self::compute_properties(projected_schema);

        Self {
            server_name_filters,
            mcp_servers,
            plan_properties,
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

impl DisplayAs for McpToolsTableExecutionPlan {
    fn fmt_as(&self, _t: DisplayFormatType, f: &mut Formatter) -> std::fmt::Result {
        write!(f, "McpToolsTableExecutionPlan")
    }
}

impl ExecutionPlan for McpToolsTableExecutionPlan {
    fn name(&self) -> &str {
        "McpToolsTableExecutionPlan"
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
        let server_configs = self.mcp_servers.iter().map(|v| v.clone()).collect();
        Ok(Box::pin(McpToolsRecordBatchStream::new(
            self.schema(),
            server_configs,
            self.server_name_filters.clone(),
        )))
    }
}

struct McpToolsRecordBatchStream {
    schema: SchemaRef,
    server_configs: Vec<Arc<McpServerConfig>>,
    server_name_filters: Option<Vec<String>>,

    polled_index: usize,
    current_server_name: Option<String>,
    current_listing_future: Option<Pin<Box<dyn Future<Output = Result<Vec<Tool>>> + Send>>>,
}

impl McpToolsRecordBatchStream {
    fn new(
        schema: SchemaRef,
        server_configs: Vec<Arc<McpServerConfig>>,
        server_name_filters: Option<Vec<String>>,
    ) -> Self {
        Self {
            schema,
            server_configs,
            server_name_filters,
            polled_index: 0,
            current_server_name: None,
            current_listing_future: None,
        }
    }
}

impl RecordBatchStream for McpToolsRecordBatchStream {
    fn schema(&self) -> SchemaRef {
        self.schema.clone()
    }
}

impl Stream for McpToolsRecordBatchStream {
    type Item = Result<RecordBatch>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        if self.current_listing_future.is_none() {
            let Some(next_config) = self.move_to_next_config() else {
                return Ready(None);
            };

            let current_config = next_config.clone();
            let future = || {
                return Box::pin(async {
                    let cli = McpClient::new(current_config);
                    cli.list_tools().await
                });
            };

            self.current_server_name = Some(next_config.name.clone());
            self.current_listing_future = Some(future());
        }

        let polled = self
            .current_listing_future
            .as_mut()
            .unwrap()
            .as_mut()
            .poll(cx);
        let tools = match polled {
            Ready(Ok(tools)) => tools,
            Ready(Err(e)) => {
                return Ready(Some(Err(e)));
            }
            Poll::Pending => return Poll::Pending,
        };
        self.current_listing_future = None;

        let tool_names: Vec<_> = tools.iter().map(|tool| tool.name.to_string()).collect();
        let descriptions: Vec<_> = tools
            .iter()
            .map(|tool| tool.description.to_string())
            .collect();
        let input_schemas: Vec<_> = tools
            .iter()
            .map(|tool| {
                serde_json::to_string(&tool.input_schema).map_or_else(|_e| None, |v| Some(v))
            })
            .collect();

        let result_arrays = HashMap::<&str, ArrayRef>::from([
            (
                "server_name",
                Arc::new(StringArray::from(vec![
                    self.current_server_name.clone();
                    tools.len()
                ])) as ArrayRef,
            ),
            ("tool_name", Arc::new(StringArray::from(tool_names))),
            ("description", Arc::new(StringArray::from(descriptions))),
            ("input_schema", Arc::new(StringArray::from(input_schemas))),
        ]);

        let mut columns = Vec::<ArrayRef>::new();
        for field in self.schema.fields.iter() {
            let ar = result_arrays.get(field.name().as_str()).unwrap().clone();
            columns.push(ar)
        }

        let Ok(rec) = RecordBatch::try_new(self.schema.clone(), columns) else {
            return Ready(Some(exec_err!(
                "failed to create RecordBatch. columns: {:?}",
                self.schema
            )));
        };

        Ready(Some(Ok(rec)))
    }
}

impl McpToolsRecordBatchStream {
    fn move_to_next_config(&mut self) -> Option<&Arc<McpServerConfig>> {
        loop {
            let Some(next_config) = self.server_configs.get(self.polled_index) else {
                return None;
            };
            self.polled_index += 1;

            let Some(filters) = &self.server_name_filters else {
                return Some(next_config);
            };

            if filters.iter().any(|f| next_config.name == f.clone()) {
                return Some(next_config);
            }
        }
    }
}
