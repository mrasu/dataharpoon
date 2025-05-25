pub(super) enum InputErr {
    InComplete,
}

pub(super) fn split_to_sqls(input: String) -> Result<Vec<String>, InputErr> {
    let mut sqls = Vec::<String>::new();
    let mut current_input = String::new();
    let mut in_single_quote = false;
    let mut in_double_quote = false;

    for c in input.chars() {
        if c == '\'' && !in_double_quote {
            in_single_quote = !in_single_quote;
        } else if c == '"' && !in_single_quote {
            in_double_quote = !in_double_quote;
        }

        if c == ';' && !in_single_quote && !in_double_quote {
            let sql = current_input.trim();
            if !sql.is_empty() {
                sqls.push(sql.to_string() + ";");
                current_input.clear();
            }
        } else {
            current_input.push(c);
        }
    }

    if in_double_quote || in_single_quote {
        return Err(InputErr::InComplete);
    }

    let remaining = current_input.trim();
    if !remaining.is_empty() {
        sqls.push(remaining.to_string() + ";");
    }

    Ok(sqls)
}
