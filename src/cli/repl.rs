use crate::agent::agent_error::AgentError;
use crate::agent::query_inference_agent::QueryInferenceAgent;
use crate::cli::helper::split_to_sqls;
use crate::cli::input_validator::ReplValidator;
use crate::cli::ui::display_content;
use crate::config::config::Config;
use crate::engine::context::Context;
use crate::model::ui::display_text::DisplayContent;
use crate::repo::mcp_repo::McpRepo;
use reedline::{DefaultPrompt, DefaultPromptSegment, Reedline, Signal};
use regex::Regex;
use std::error::Error;
use std::io;
use std::io::Write;
use std::rc::Rc;
use std::sync::LazyLock;
use std::time::Duration;

static ASK_COMMAND_REGEX: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"/ask\s+(\S.+)").unwrap());

pub async fn run_repl(config: Config) {
    let validator = Box::new(ReplValidator {});
    let mut line_editor = Reedline::create().with_validator(validator);
    let prompt = DefaultPrompt::new(
        DefaultPromptSegment::Basic("DataHarpoon ".to_string()),
        DefaultPromptSegment::Empty,
    );

    let ctx = Rc::new(Context::new(config.clone()));

    loop {
        flush_stdout().await;

        let sig = line_editor.read_line(&prompt);
        match sig {
            Ok(Signal::Success(text)) => {
                if text.starts_with("/") {
                    // TODO: change mode to be able to talk with AI
                    if let Some(captures) = ASK_COMMAND_REGEX.captures(text.as_str()) {
                        // e.g.
                        // let question = "please tell me the number of users (written in example/user.csv) with the role 'member' in each organization (written in example/org.json).";
                        // let question = "サンフランシスコの現在時間を知りたい";
                        // let question = "githubのgithub/github-mcp-serverリポジトリのissueを10個挙げてください";
                        // let question = "githubのgithub/github-mcp-serverリポジトリのissueの一覧を3個取って、タイトルを基に、claudeを使ってbug,feature-request,otherで分類して";

                        let question = captures.get(1).unwrap().as_str();
                        match run_ask_command(ctx.clone(), &config, question).await {
                            Ok(_) => {}
                            Err(e) => handle_error(&e),
                        }
                    } else {
                        println!(
                            "invalid slash commands. {}\nonly /ask is supported now.",
                            text
                        )
                    }
                } else if let Ok(sqls) = split_to_sqls(text.clone()) {
                    for sql in sqls {
                        let res = ctx.run_sql(sql.as_str()).await;
                        match res {
                            Ok(df) => {
                                if let Err(e) = df.show().await {
                                    handle_error(&e)
                                }
                            }
                            Err(e) => handle_error(&e),
                        }
                    }
                } else {
                    panic!("invalid input. {}", text);
                };
            }
            Ok(Signal::CtrlC) | Ok(Signal::CtrlD) => {
                println!("Aborted");
                break;
            }
            Err(err) => {
                println!("Error while reading lines: {:?}", err);
            }
        }
    }
}

async fn run_ask_command(
    ctx: Rc<Context>,
    config: &Config,
    question: &str,
) -> Result<(), AgentError> {
    let agent = if config.dev.use_mock {
        QueryInferenceAgent::new_mocked(ctx.clone(), &config)
    } else {
        let repo = McpRepo::new(ctx.clone());
        let mcp_tools = repo.list_mcp_tools().await?;
        QueryInferenceAgent::new(ctx.clone(), &config, mcp_tools)
    };

    let fn_display = |contents: Vec<DisplayContent>| {
        for c in contents {
            display_content(c)
        }
    };

    agent.run_inference_loop(question, fn_display).await?;

    Ok(())
}

async fn flush_stdout() {
    let mut stdout = io::stdout();
    stdout.flush().unwrap();

    // Wait briefly to ensure stdout has fully flushed to avoid collisions with CPR.
    tokio::time::sleep(Duration::from_millis(100)).await;
}

fn handle_error(e: &dyn Error) {
    println!("Error! {}", e);
}
