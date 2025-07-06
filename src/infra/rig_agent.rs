use async_trait::async_trait;
use rig::agent::Agent;
use rig::completion::{Chat, Message, PromptError};
use rig::providers::anthropic::completion::CompletionModel;

#[async_trait]
pub trait RigAgent {
    async fn chat(&mut self, message: &str) -> Result<String, PromptError>;
}

pub struct RigAgentImpl {
    agent: Agent<CompletionModel>,
    chat_history: Vec<Message>,
}

impl RigAgentImpl {
    pub fn new(agent: Agent<CompletionModel>) -> Self {
        Self {
            agent,
            chat_history: Vec::new(),
        }
    }
}

#[async_trait]
impl RigAgent for RigAgentImpl {
    async fn chat(&mut self, message: &str) -> Result<String, PromptError> {
        let response = self.agent.chat(message, self.chat_history.clone()).await?;

        self.chat_history.push(Message::user(message));
        self.chat_history.push(Message::assistant(response.clone()));

        Ok(response)
    }
}
