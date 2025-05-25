use crate::cli::helper::split_to_sqls;
use crate::cli::input_validator::ReplValidator;
use reedline::{DefaultPrompt, DefaultPromptSegment, Reedline, Signal};

pub fn run_repl() {
    let validator = Box::new(ReplValidator {});
    let mut line_editor = Reedline::create().with_validator(validator);
    let prompt = DefaultPrompt::new(
        DefaultPromptSegment::Basic("DataHarpoon ".to_string()),
        DefaultPromptSegment::Empty,
    );

    loop {
        let sig = line_editor.read_line(&prompt);
        match sig {
            Ok(Signal::Success(text)) => {
                if let Ok(sqls) = split_to_sqls(text.clone()) {
                    println!("SQL:");
                    for sql in sqls {
                        println!("--\n{}", sql);
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
