use std::fmt::Debug;
use std::fs;

use html_parser::prelude::*;

pub mod filter;
pub mod find;
pub mod full;
pub mod strings;

fn handle_auto_closing(html: &str) -> String {
    let mut output = String::with_capacity(html.len());
    let mut tag_name = String::new();
    let mut reading = false;
    let mut last_slash = false;
    for ch in html.chars() {
        if ch == '>' && last_slash {
            output.push_str("> </");
            output.push_str(&tag_name);
            output.push('>');
            tag_name.clear();
            last_slash = false;
            reading = false;
        } else if ch == '/' {
            if last_slash {
                output.push('/');
            } else {
                last_slash = true;
            }
            reading = false;
        } else {
            if last_slash {
                last_slash = false;
                output.push('/');
            }
            if ch == '<' {
                reading = true;
                tag_name.clear();
                output.push('<');
            } else {
                if ch.is_whitespace() || ch == '!' || ch == '>' {
                    reading = false;
                }
                output.push(ch);
                if reading {
                    tag_name.push(ch);
                }
            }
        }
    }
    output
}

fn format_html(html: &str) -> String {
    let mut formatted = html
        .replace('/', " /")
        .replace('\n', " ")
        .replace("< ", "<")
        .replace(" >", ">")
        .replace("<", " <")
        .replace(">", "> ");
    loop {
        let out = formatted.replace("  ", " ");
        if out == formatted {
            break;
        }
        formatted = out;
    }
    handle_auto_closing(&formatted).replace(" >", ">")
}

fn test_maker<T: Debug>(name: &str, expected: &str, output: Html, msg: T) {
    let formatted_input = format_html(expected);
    let formatted_output = format_html(&output.to_string());
    if formatted_output != formatted_input {
        let output_path = format!("output.{}.html", name);
        let expected_path = format!("expected.{}.html", name);
        fs::write(&output_path, formatted_output.replace(' ', "\n"))
            .expect("Permission denied: failed to write to directory.");
        fs::write(&expected_path, formatted_input.replace(' ', "\n"))
            .expect("Permission denied: failed to write to directory.");
        panic!(
            "Error occurred.\n{msg:?}\nOutput:\n--------------------\n{formatted_output}\n--------------------\nUse `diff {output_path} {expected_path}` to see the problem."
        );
    }
}
