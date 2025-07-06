use crate::agent::agent_error::AgentError;
use crate::agent::response::Response;
use crate::engine::context::Context;
use crate::infra::rig_agent::RigAgent;
use crate::util::arrow::json::convert_to_json;
use datafusion::arrow::record_batch::RecordBatch;
use std::rc::Rc;

#[derive(Debug)]
pub(super) enum State {
    Initial(InitialState),
    RunQuery(RunQueryState),
    Chat(ChatState),
    ChatWithHuman(ChatWithHumanState),
    AttemptCompletion(AttemptCompletionState),
}

#[derive(Debug)]
pub(super) struct InitialState {}

impl InitialState {
    pub fn new() -> Self {
        Self {}
    }
}

impl InitialState {
    pub async fn proceed(
        &self,
        agent: &mut Box<dyn RigAgent>,
        user_input: &str,
    ) -> Result<Response, AgentError> {
        let first_message = format!("<objective>{user_input}</objective>").to_string();
        let response = agent.chat(first_message.as_str()).await?;

        let resp = Response::parse(response.as_str())?;
        Ok(resp)
    }
}

#[derive(Debug)]
pub(super) struct RunQueryState {
    query: String,
}

impl RunQueryState {
    pub fn new(query: String) -> Self {
        Self { query }
    }

    pub async fn run(&self, ctx: Rc<Context>) -> Result<Response, AgentError> {
        let df = ctx.run_sql(&self.query).await?;
        let batch = df.collect().await?;

        let resp_json_u8 = convert_to_json(&batch)
            .await
            .map_err(|e| AgentError::new_unexpected_error(Box::new(e)))?;
        let resp_json = String::from_utf8(resp_json_u8)
            .map_err(|e| AgentError::new_unexpected_error(Box::new(e)))?;

        let next_action = ChatState::new(format!("Result: {}", resp_json));
        let response = Response::new(resp_json, State::Chat(next_action), vec![]);
        Ok(response)
    }
}

#[derive(Debug)]
pub(super) struct ChatState {
    message: String,
}

impl ChatState {
    pub fn new(message: String) -> Self {
        Self { message }
    }

    pub async fn chat(
        &self,
        agent: &mut Box<dyn RigAgent>,
    ) -> Result<(Response, usize), AgentError> {
        let response = agent.chat(self.message.as_str()).await?;

        let resp = Response::parse(response.as_str())?;
        Ok((resp, 1))
    }
}

#[derive(Debug)]
pub(super) struct ChatWithHumanState {}

impl ChatWithHumanState {
    pub fn new() -> Self {
        Self {}
    }

    pub async fn chat(
        &self,
        agent: &mut Box<dyn RigAgent>,
        message: &str,
    ) -> Result<(Response, usize), AgentError> {
        let response = agent.chat(message).await?;

        let resp = Response::parse(response.as_str())?;
        Ok((resp, 1))
    }
}

#[derive(Debug)]
pub(super) struct AttemptCompletionState {
    pub query: String,
}

impl AttemptCompletionState {
    pub fn new(query: String) -> Self {
        Self { query }
    }

    pub async fn preview_query(&self, ctx: Rc<Context>) -> Result<Vec<RecordBatch>, AgentError> {
        let df = ctx.run_sql(&self.query).await?;
        let batch = df.limit(0, Some(5))?.collect().await?;

        Ok(batch)
    }
}
