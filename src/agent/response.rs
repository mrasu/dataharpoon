use crate::agent::agent_error::AgentError;
use crate::agent::state::{AttemptCompletionState, RunQueryState, State};
use crate::model::ui::display_text::{AttemptCompletion, DisplayContent, Raw, RunQuery, Thinking};
use regex::Regex;
use std::sync::LazyLock;

#[derive(Debug)]
pub(super) struct Response {
    pub(super) raw: String,
    pub next_state: State,
    pub contents: Vec<ResponseContent>,
}

#[derive(Debug, Clone)]
pub enum ResponseContent {
    Raw(RawText),
    Thinking(ThinkingText),
    RunQuery(RunQueryTool),
    AttemptCompletion(AttemptCompletionTool),
}

#[derive(Debug, Clone)]
pub struct RawText {
    pub text: String,
}

#[derive(Debug, Clone)]
pub struct ThinkingText {
    pub text: String,
}

#[derive(Debug, Clone)]
pub struct RunQueryTool {
    pub query: String,
}

#[derive(Debug, Clone)]
pub struct AttemptCompletionTool {
    pub query: String,
}

static START_TAG_REGEX: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"<([^/>]+)>").unwrap());

impl Response {
    pub fn new(raw: String, next_action: State, contents: Vec<ResponseContent>) -> Self {
        Self {
            raw,
            next_state: next_action,
            contents,
        }
    }

    pub fn parse(text: &str) -> Result<Self, AgentError> {
        let mut remaining_text = text;
        let mut found_contents: Vec<ResponseContent> = vec![];

        loop {
            let (content, end_pos) = Self::parse_one(remaining_text);
            found_contents.push(content);

            remaining_text = remaining_text[end_pos..].trim_start();
            if remaining_text.is_empty() {
                break;
            }
        }

        let mut contents = Vec::<ResponseContent>::new();
        let mut next_state: Option<State> = None;
        for content in found_contents {
            match content {
                ResponseContent::RunQuery(tool) => {
                    next_state = Some(State::RunQuery(RunQueryState::new(tool.query.to_string())));
                    break;
                }
                ResponseContent::AttemptCompletion(tool) => {
                    next_state = Some(State::AttemptCompletion(AttemptCompletionState::new(
                        tool.query,
                    )));
                    break;
                }
                _ => {
                    contents.push(content.clone());
                }
            }
        }

        let Some(next_action) = next_state else {
            return Err(AgentError::new_no_tool_included_error(text.to_string()));
        };

        Ok(Self {
            raw: text.to_string(),
            contents,
            next_state: next_action,
        })
    }

    fn parse_one(text: &str) -> (ResponseContent, usize) {
        let Some(start) = START_TAG_REGEX.captures(text) else {
            return (ResponseContent::Raw(RawText::new(text)), text.len());
        };

        let tag_name = start.get(1).unwrap().as_str();
        let start_pos = start.get(0).unwrap().start();

        let before_start_text = text[..start_pos].trim();
        if before_start_text.len() > 0 {
            return (
                ResponseContent::Raw(RawText::new(before_start_text)),
                start_pos,
            );
        }

        let start_tag_end_pos = start.get(0).unwrap().end();

        let end_tag = format!("</{}>", tag_name);

        let Some(end_pos) = text[start_pos..].find(&end_tag) else {
            return (ResponseContent::Raw(RawText::new(text)), text.len());
        };

        let end_tag_start_pos = start_pos + end_pos;
        let tool_text = &text[start_tag_end_pos..end_tag_start_pos];
        let next_read_start_pos = end_tag_start_pos + end_tag.len();
        match tag_name {
            "run_query" => {
                let tool = RunQueryTool::try_new(tool_text)
                    .map(|t| ResponseContent::RunQuery(t))
                    .unwrap_or(ResponseContent::Raw(RawText::new(text)));
                (tool, next_read_start_pos)
            }
            "attempt_completion" => {
                let tool = AttemptCompletionTool::try_new(tool_text)
                    .map(|t| ResponseContent::AttemptCompletion(t))
                    .unwrap_or(ResponseContent::Raw(RawText::new(text)));
                (tool, next_read_start_pos)
            }
            "thinking" => (
                ResponseContent::Thinking(ThinkingText::new(tool_text)),
                next_read_start_pos,
            ),
            _ => (
                ResponseContent::Raw(RawText::new(&text[0..next_read_start_pos])),
                next_read_start_pos,
            ),
        }
    }
}

impl From<&ResponseContent> for DisplayContent {
    fn from(value: &ResponseContent) -> Self {
        match value {
            ResponseContent::Raw(raw) => raw.into(),
            ResponseContent::Thinking(thinking) => thinking.into(),
            ResponseContent::RunQuery(run_query) => run_query.into(),
            ResponseContent::AttemptCompletion(attempt) => attempt.into(),
        }
    }
}

impl RawText {
    fn new(text: &str) -> Self {
        RawText {
            text: text.to_string(),
        }
    }
}

impl From<&RawText> for DisplayContent {
    fn from(value: &RawText) -> Self {
        DisplayContent::Raw(Raw {
            text: value.text.clone(),
        })
    }
}

static RUN_QUERY_QUERY_TAG_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?s)<query>(.+?)</query>").unwrap());

impl RunQueryTool {
    fn try_new(text: &str) -> Option<Self> {
        let captures = RUN_QUERY_QUERY_TAG_REGEX.captures(text)?;

        let tool = Self {
            query: captures.get(1).unwrap().as_str().to_string(),
        };
        Some(tool)
    }
}

impl From<&RunQueryTool> for DisplayContent {
    fn from(value: &RunQueryTool) -> Self {
        DisplayContent::RunQuery(RunQuery {
            query: value.query.clone(),
        })
    }
}

static ATTEMPT_COMPLETION_QUERY_TAG_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?s)<query>(.+?)</query>").unwrap());

impl AttemptCompletionTool {
    fn try_new(text: &str) -> Option<Self> {
        let query_captures = ATTEMPT_COMPLETION_QUERY_TAG_REGEX.captures(text)?;

        let tool = Self {
            query: query_captures.get(1).unwrap().as_str().to_string(),
        };
        Some(tool)
    }
}

impl From<&AttemptCompletionTool> for DisplayContent {
    fn from(value: &AttemptCompletionTool) -> Self {
        DisplayContent::AttemptCompletion(AttemptCompletion {
            query: value.query.clone(),
            preview_batch: vec![],
        })
    }
}

impl ThinkingText {
    fn new(text: &str) -> Self {
        Self {
            text: text.to_string(),
        }
    }
}

impl From<&ThinkingText> for DisplayContent {
    fn from(value: &ThinkingText) -> Self {
        DisplayContent::Thinking(Thinking {
            text: value.text.clone(),
        })
    }
}
