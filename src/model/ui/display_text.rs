use datafusion::arrow::array::RecordBatch;

#[derive(Debug, Clone)]
pub enum DisplayContent {
    Raw(Raw),
    Thinking(Thinking),
    RunQuery(RunQuery),
    AttemptCompletion(AttemptCompletion),
}

#[derive(Debug, Clone)]
pub struct Raw {
    pub text: String,
}

#[derive(Debug, Clone)]
pub struct Thinking {
    pub text: String,
}

#[derive(Debug, Clone)]
pub struct RunQuery {
    pub query: String,
}

#[derive(Debug, Clone)]
pub struct AttemptCompletion {
    pub query: String,
    pub preview_batch: Vec<RecordBatch>,
}
