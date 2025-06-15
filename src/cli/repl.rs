use crate::cli::helper::split_to_sqls;
use crate::cli::input_validator::ReplValidator;
use crate::config::config::Config;
use crate::engine::context::Context;
use reedline::{DefaultPrompt, DefaultPromptSegment, Reedline, Signal};
use std::error::Error;
use std::io;
use std::io::Write;
use std::time::Duration;

pub async fn run_repl(config: Config) {
    let validator = Box::new(ReplValidator {});
    let mut line_editor = Reedline::create().with_validator(validator);
    let prompt = DefaultPrompt::new(
        DefaultPromptSegment::Basic("DataHarpoon ".to_string()),
        DefaultPromptSegment::Empty,
    );

    let ctx = Context::new(config);

    loop {
        flush_stdout().await;

        let sig = line_editor.read_line(&prompt);
        match sig {
            Ok(Signal::Success(text)) => {
                if let Ok(sqls) = split_to_sqls(text.clone()) {
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

async fn flush_stdout() {
    let mut stdout = io::stdout();
    stdout.flush().unwrap();

    // Wait briefly to ensure stdout has fully flushed to avoid collisions with CPR.
    tokio::time::sleep(Duration::from_millis(100)).await;
}

fn handle_error(e: &dyn Error) {
    println!("Error happened. {}", e);
}
