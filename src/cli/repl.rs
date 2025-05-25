use crate::cli::helper::split_to_sqls;
use crate::cli::input_validator::ReplValidator;
use crate::config::config::Config;
use crate::engine::context::Context;
use reedline::{DefaultPrompt, DefaultPromptSegment, Reedline, Signal};
use std::error::Error;

pub async fn run_repl(config: Config) {
    let validator = Box::new(ReplValidator {});
    let mut line_editor = Reedline::create().with_validator(validator);
    let prompt = DefaultPrompt::new(
        DefaultPromptSegment::Basic("DataHarpoon ".to_string()),
        DefaultPromptSegment::Empty,
    );

    let ctx = Context::new(config);

    loop {
        let sig = line_editor.read_line(&prompt);
        match sig {
            Ok(Signal::Success(text)) => {
                if let Ok(sqls) = split_to_sqls(text.clone()) {
                    println!("SQL:");
                    for sql in sqls {
                        println!("--\n{}", sql);
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
                println!("Error: {:?}", err);
            }
        }
    }
}

fn handle_error(e: &dyn Error) {
    println!("{}", e);
}
