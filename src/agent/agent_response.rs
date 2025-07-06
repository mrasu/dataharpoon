use crate::agent::response::ResponseContent;
use crate::model::ui::display_text::DisplayContent;

pub struct AgentResponse {
    pub display_contents: Vec<DisplayContent>,
    pub chat_count: usize,
    pub continues: bool,
}
impl Default for AgentResponse {
    fn default() -> Self {
        Self {
            display_contents: vec![],
            chat_count: 0,
            continues: true,
        }
    }
}

impl AgentResponse {
    pub fn new_with_contents(response_contents: Vec<ResponseContent>) -> Self {
        Self {
            display_contents: Self::display_contents(response_contents),
            ..Default::default()
        }
    }

    pub fn new_for_chat(response_contents: Vec<ResponseContent>, chat_count: usize) -> Self {
        Self {
            display_contents: Self::display_contents(response_contents),
            chat_count,
            ..Default::default()
        }
    }

    pub fn new_for_finisher(content: DisplayContent) -> Self {
        Self {
            display_contents: vec![content],
            continues: false,
            ..Default::default()
        }
    }

    fn display_contents(response_contents: Vec<ResponseContent>) -> Vec<DisplayContent> {
        response_contents
            .iter()
            .map(|content| content.into())
            .collect()
    }
}
