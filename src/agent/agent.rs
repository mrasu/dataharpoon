use crate::agent::agent_error::AgentError;
use crate::agent::agent_response::AgentResponse;
use crate::agent::state::{ChatWithHumanState, InitialState, State};
use crate::engine::context::Context;
use crate::infra::rig_agent::RigAgent;
use crate::model::ui::display_text::{AttemptCompletion, DisplayContent};
use log::info;
use std::rc::Rc;

pub(super) struct Agent {
    rig_agent: Box<dyn RigAgent>,
    engine_context: Rc<Context>,
    current_state: State,
}

impl Agent {
    pub fn new(rig_agent: Box<dyn RigAgent>, engine_context: Rc<Context>) -> Self {
        Self {
            rig_agent,
            engine_context,
            current_state: State::Initial(InitialState::new()),
        }
    }

    pub async fn proceed(&mut self, user_input: Option<&str>) -> Result<AgentResponse, AgentError> {
        info!("proceed. input: {:?}", user_input);

        match &self.current_state {
            State::Initial(state) => {
                info!("State::Initial: {:?}", state);
                let Some(user_input) = user_input else {
                    return Err(AgentError::new_no_input_error("no question provided"));
                };
                let response = state.proceed(&mut self.rig_agent, user_input).await?;

                info!("raw_response: {:?}\n", response.raw);

                self.current_state = response.next_state;
                Ok(AgentResponse::new_with_contents(response.contents))
            }
            State::RunQuery(state) => {
                info!("State::RunQuery: {:?}", state);
                let response = state.run(self.engine_context.clone()).await?;

                info!("raw_response: {:?}\n", response.raw);

                self.current_state = response.next_state;
                Ok(AgentResponse::new_with_contents(response.contents))
            }
            State::Chat(state) => {
                info!("State::Chat: {:?}\n", state);
                let (response, chat_count) = state.chat(&mut self.rig_agent).await?;

                info!("raw_response: {:?}\n", response.raw);

                self.current_state = response.next_state;
                Ok(AgentResponse::new_for_chat(response.contents, chat_count))
            }
            State::ChatWithHuman(state) => {
                info!("State::ChatWithHuman: {:?}\n", state);
                let Some(user_input) = user_input else {
                    return Err(AgentError::new_no_input_error("no question provided"));
                };
                let (response, chat_count) = state.chat(&mut self.rig_agent, user_input).await?;

                info!("raw_response: {:?}\n", response.raw);

                self.current_state = response.next_state;
                Ok(AgentResponse::new_for_chat(response.contents, chat_count))
            }
            State::AttemptCompletion(state) => {
                info!("State::AttemptCompletion: {:?}", state);

                let preview_batch = state.preview_query(self.engine_context.clone()).await?;
                let content = DisplayContent::AttemptCompletion(AttemptCompletion {
                    query: state.query.clone(),
                    preview_batch,
                });

                self.current_state = State::ChatWithHuman(ChatWithHumanState::new());
                Ok(AgentResponse::new_for_finisher(content))
            }
        }
    }
}
