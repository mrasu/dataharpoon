use crate::model::ui::display_text::DisplayContent;
use crossterm::style::Print;
use crossterm::{
    ExecutableCommand,
    style::{Color, PrintStyledContent, Stylize},
};
use datafusion::arrow::util::pretty;
use std::io::stdout;

pub(super) fn display_content(response_content: DisplayContent) {
    match response_content {
        DisplayContent::Raw(ref text) => println!("\n{}", text.text.trim()),
        DisplayContent::Thinking(ref thinking) => {
            let mut stdout = stdout();

            stdout.execute(Print("thinking:\n")).unwrap();
            stdout
                .execute(PrintStyledContent(thinking.text.trim().with(Color::Rgb {
                    r: 128,
                    g: 128,
                    b: 128,
                })))
                .unwrap();
            println!();
        }
        DisplayContent::RunQuery(ref tool) => {
            let mut stdout = stdout();

            stdout.execute(Print("Run SQL:\n> ")).unwrap();
            stdout.execute(Print(tool.query.trim())).unwrap();
            println!();
        }
        DisplayContent::AttemptCompletion(ref tool) => {
            let mut stdout = stdout();

            stdout
                .execute(Print("Completed! suggested SQL is :\n "))
                .unwrap();
            stdout.execute(Print(tool.query.trim())).unwrap();

            stdout.execute(Print("\nPreview:\n")).unwrap();
            pretty::print_batches(&tool.preview_batch).unwrap();
            println!();
        }
    }
}
