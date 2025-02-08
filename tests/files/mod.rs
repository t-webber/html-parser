use std::fmt::Debug;
use std::fs;

use html_parser::prelude::*;

pub mod filter;
pub mod find;
pub mod full;

fn format_html(html: &str) -> String {
    let mut old = html
        .replace('/', " /")
        .replace('\n', " ")
        .replace("< ", "<")
        .replace(" >", ">")
        .replace("<", " <")
        .replace(">", "> ");
    loop {
        let out = old.replace("  ", " ");
        if out == old {
            break;
        }
        old = out;
    }
    old
}

fn test_maker<T: Debug>(name: &str, expected: &str, output: Html, msg: T) {
    let formatted_input = format_html(expected);
    let formatted_output = format_html(&output.to_string());
    if formatted_output != formatted_input {
        let output_path = format!("output.{}.html", name);
        let expected_path = format!("expected.{}.html", name);
        fs::write(&output_path, &formatted_output)
            .expect("Permission denied: failed to write to directory.");
        fs::write(&expected_path, &formatted_input)
            .expect("Permission denied: failed to write to directory.");
        panic!(
            "Error occurred.\n{msg:?}\nOutput:\n--------------------\n{formatted_output}\n--------------------\nUse `diff {output_path} {expected_path}` to see the problem."
        );
    }
}
