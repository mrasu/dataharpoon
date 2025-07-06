use datafusion::common::DataFusionError;
use rig::completion::PromptError;
use std::fmt;
use std::fmt::{Debug, Display, Formatter};

#[derive(Debug)]
pub enum AgentError {
    PromptError(PromptError),
    NoToolIncludedResponseError(NoToolIncludedResponseError),
    DataFusionError(DataFusionError),
    NoInputError(NoInputError),
    UnexpectedError(Box<dyn std::error::Error>),
}

impl Display for AgentError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            AgentError::PromptError(err) => Display::fmt(err, f),
            AgentError::NoToolIncludedResponseError(err) => Display::fmt(err, f),
            AgentError::DataFusionError(err) => Display::fmt(err, f),
            AgentError::NoInputError(err) => Display::fmt(err, f),
            AgentError::UnexpectedError(err) => Display::fmt(err, f),
        }
    }
}
impl std::error::Error for AgentError {}

impl AgentError {
    pub fn new_no_input_error(message: &str) -> AgentError {
        AgentError::NoInputError(NoInputError {
            message: message.to_string(),
        })
    }

    pub fn new_no_tool_included_error(response: String) -> AgentError {
        AgentError::NoToolIncludedResponseError(NoToolIncludedResponseError { response })
    }

    pub fn new_unexpected_error(err: Box<dyn std::error::Error>) -> AgentError {
        AgentError::UnexpectedError(err)
    }
}

impl From<PromptError> for AgentError {
    fn from(err: PromptError) -> Self {
        AgentError::PromptError(err)
    }
}

impl From<DataFusionError> for AgentError {
    fn from(err: DataFusionError) -> Self {
        AgentError::DataFusionError(err)
    }
}

#[derive(Debug)]
pub struct NoInputError {
    message: String,
}

impl Display for NoInputError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "No input provided. {}", self.message)
    }
}

#[derive(Debug)]
pub struct NoToolIncludedResponseError {
    response: String,
}

impl Display for NoToolIncludedResponseError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "NoToolIncluded in response. response: {}", self.response)
    }
}
