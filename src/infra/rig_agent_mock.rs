use crate::infra::rig_agent::RigAgent;
use async_trait::async_trait;
use rig::completion::{Message, PromptError};

pub struct RigAgentMock {
    chat_history: Vec<Message>,
    question: Option<String>,
}

impl RigAgentMock {
    pub fn new() -> Self {
        Self {
            chat_history: Vec::new(),
            question: None,
        }
    }
}

#[async_trait]
impl RigAgent for RigAgentMock {
    async fn chat(&mut self, message: &str) -> Result<String, PromptError> {
        let response = self.next_chat(message);

        self.chat_history.push(Message::user(message));
        self.chat_history
            .push(Message::assistant(response.to_owned()));
        Ok(response.to_owned())
    }
}

const QUESTION_KEY_FOR_SAN_FRANCISCO_TIME: &str = "サンフランシスコ";
const QUESTION_KEY_FOR_EXAMPLE_FILES: &str = "user.csv";

impl RigAgentMock {
    fn next_chat(&mut self, message: &str) -> String {
        if self.question.is_none() {
            self.question = Some(message.to_string());
        }
        let question = self.question.clone().unwrap_or("".to_string());

        if question.contains(QUESTION_KEY_FOR_SAN_FRANCISCO_TIME) {
            return Self::next_chat_for_san_francisco(message);
        } else if question.contains(QUESTION_KEY_FOR_EXAMPLE_FILES) {
            return Self::next_chat_for_example_files(message);
        }

        format!("not registered question: {message:?}").to_string()
    }
}

const SAN_FRANCISCO_TIME_1_INPUT_SCHEMA: &str =
    include_str!("./rig_mock_responses/san_francisco_time/1_input_schema.txt");
const SAN_FRANCISCO_TIME_2_EXEC_MCP: &str =
    include_str!("./rig_mock_responses/san_francisco_time/2_exec_mcp.txt");
const SAN_FRANCISCO_TIME_3_ATTEMPT_COMPLETION: &str =
    include_str!("./rig_mock_responses/san_francisco_time/3_attempt_completion.txt");

impl RigAgentMock {
    fn next_chat_for_san_francisco(message: &str) -> String {
        if message.contains("<objective>") {
            return SAN_FRANCISCO_TIME_1_INPUT_SCHEMA.to_string();
        }

        if message.contains("input_schema") {
            return SAN_FRANCISCO_TIME_2_EXEC_MCP.to_string();
        }

        if message.contains(r#"current_time":"{\n  \"timezone"#) {
            return SAN_FRANCISCO_TIME_3_ATTEMPT_COMPLETION.to_string();
        }

        format!("not registered message: {message:?}").to_string()
    }
}

const EXAMPLE_FILES_1_DESCRIBE_USER: &str =
    include_str!("./rig_mock_responses/example_files/1_describe_user.txt");
const EXAMPLE_FILES_2_DESCRIBE_ORG: &str =
    include_str!("./rig_mock_responses/example_files/2_describe_org.txt");
const EXAMPLE_FILES_3_PREVIEW_USER: &str =
    include_str!("./rig_mock_responses/example_files/3_preview_user.txt");
const EXAMPLE_FILES_4_PREVIEW_ORG: &str =
    include_str!("./rig_mock_responses/example_files/4_preview_org.txt");
const EXAMPLE_FILES_5_ATTEMPT_COMPLETION: &str =
    include_str!("./rig_mock_responses/example_files/5_attempt_completion.txt");

impl RigAgentMock {
    fn next_chat_for_example_files(message: &str) -> String {
        if message.contains("<objective>") {
            return EXAMPLE_FILES_1_DESCRIBE_USER.to_string();
        }

        if message.contains("{\"column_name\":\"user_name\",\"data_type\":\"Utf8\"") {
            return EXAMPLE_FILES_2_DESCRIBE_ORG.to_string();
        }

        if message.contains("{\"column_name\":\"industry\",\"data_type\":\"Utf8\"") {
            return EXAMPLE_FILES_3_PREVIEW_USER.to_string();
        }

        if message.contains("[{\"id\":1,\"organization_id\":1001,\"user_name\":\"john_doe\"") {
            return EXAMPLE_FILES_4_PREVIEW_ORG.to_string();
        }

        if message.contains("{\"created_at\":\"2018-06-12\",\"id\":1001,\"industry\":\"Software\"")
        {
            return EXAMPLE_FILES_5_ATTEMPT_COMPLETION.to_string();
        }

        format!("not registered message: {message:?}").to_string()
    }
}
