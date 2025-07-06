use crate::cli::helper::{InputErr, split_to_sqls};
use reedline::{ValidationResult, Validator};

pub struct ReplValidator {}

impl Validator for ReplValidator {
    fn validate(&self, line: &str) -> ValidationResult {
        if line.starts_with("/") {
            ValidationResult::Complete
        } else if line.trim().ends_with(";") {
            match split_to_sqls(line.to_string()) {
                Ok(_) => ValidationResult::Complete,
                Err(InputErr::InComplete) => ValidationResult::Incomplete,
            }
        } else {
            ValidationResult::Incomplete
        }
    }
}
