use crate::agent::agent::Agent;
use crate::agent::agent_error::AgentError;
use crate::config::config::Config;
use crate::engine::context::Context;
use crate::infra::rig_agent::RigAgentImpl;
use crate::infra::rig_agent_mock::RigAgentMock;
use crate::model::engine::mcp_tool::McpTool;
use crate::model::ui::display_text::DisplayContent;
use log::info;
use rig::client::CompletionClient;
use rig::providers::anthropic;
use std::rc::Rc;
use std::time::Duration;
use tokio::time::sleep;

const SYSTEM_PROMPT_TEMPLATE: &str = include_str!("../data/prompt.md");
const SYSTEM_PROMPT_TEMPLATE_AVAILABLE_MCP_TOOL_MARK: &str = "{AVAILABLE_MCP_TOOL_PROMPT}";

const AGENT_MODEL: &str = "claude-sonnet-4-20250514";
const AGENT_TEMPERATURE: f64 = 0.8;
const AGENT_MAX_TOKENS: u64 = 10000;

pub struct QueryInferenceAgent {
    agent: Agent,
    max_prompt_count: usize,
}

impl QueryInferenceAgent {
    pub fn new(ctx: Rc<Context>, config: &Config, mcp_tools: Vec<McpTool>) -> QueryInferenceAgent {
        let system_prompt = Self::build_system_prompt(mcp_tools);
        info!("SYSTEM_PROMPT: {}", system_prompt);

        let client = anthropic::ClientBuilder::new(config.claude_token.as_str()).build();

        let rig_agent = client
            .agent(AGENT_MODEL)
            .preamble(system_prompt.as_str())
            .temperature(AGENT_TEMPERATURE)
            .max_tokens(AGENT_MAX_TOKENS)
            .build();
        let rig_agent_impl = RigAgentImpl::new(rig_agent);
        let agent = Agent::new(Box::new(rig_agent_impl), ctx.clone());

        QueryInferenceAgent {
            agent,
            max_prompt_count: config.max_prompt_count,
        }
    }

    fn build_system_prompt(mcp_tools: Vec<McpTool>) -> String {
        let tool_prompt = mcp_tools
            .iter()
            .map(|tool| {
                let desc_text = if tool.description.contains("\n") {
                    format!("<desc>{}</desc>", tool.description)
                } else {
                    tool.description.clone()
                };

                format!(
                    r#"- Server name: {}
Tool name: {}
Description: {}"#,
                    tool.server_name, tool.tool_name, desc_text
                )
            })
            .collect::<Vec<_>>()
            .join("\n");

        let system_prompt = SYSTEM_PROMPT_TEMPLATE.replace(
            SYSTEM_PROMPT_TEMPLATE_AVAILABLE_MCP_TOOL_MARK,
            tool_prompt.as_str(),
        );

        system_prompt
    }

    pub fn new_mocked(ctx: Rc<Context>, config: &Config) -> QueryInferenceAgent {
        let mock_rig_agent = RigAgentMock::new();
        let agent = Agent::new(Box::new(mock_rig_agent), ctx.clone());

        QueryInferenceAgent {
            agent,
            max_prompt_count: config.max_prompt_count,
        }
    }
}

impl QueryInferenceAgent {
    pub async fn run_inference_loop(
        mut self,
        question: &str,
        fn_display: fn(Vec<DisplayContent>),
    ) -> Result<(), AgentError> {
        let response = self.agent.proceed(Some(question.trim())).await?;
        fn_display(response.display_contents);
        if !response.continues {
            return Ok(());
        }

        let mut chat_total_count = response.chat_count;
        let mut sleep_chat_counter = 0;
        loop {
            if sleep_chat_counter > 2 {
                // TODO: Support Retry-After
                info!("sleep 30 seconds...");
                sleep(Duration::from_secs(30)).await;
                sleep_chat_counter = 0;
            }
            // TODO: handle NoToolIncludedResponseError. Ask AI sometimes to include tool.
            let response = self.agent.proceed(None).await?;
            fn_display(response.display_contents);
            sleep_chat_counter += response.chat_count;
            chat_total_count += response.chat_count;
            if !response.continues {
                break;
            }
            if self.max_prompt_count < chat_total_count {
                break;
            }
        }

        Ok(())
    }
}
