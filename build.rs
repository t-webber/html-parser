use std::fs;
use std::path::PathBuf;

fn get_files(path: PathBuf) -> Vec<PathBuf> {
    let mut res = vec![];
    let files = path.read_dir().unwrap();
    for file in files {
        let file = file.unwrap();
        if file.file_type().unwrap().is_dir() {
            res.extend(get_files(file.path()))
        } else {
            res.push(file.path());
        }
    }
    res
}

#[derive(Debug)]
struct CodeLine {
    file_id: usize,
    line_nb: usize,
    line: String,
}

fn main() {
    let files = get_files(PathBuf::from("src"));
    let source_code = files
        .iter()
        .enumerate()
        .flat_map(|(file_id, file_path)| {
            fs::read_to_string(file_path)
                .unwrap()
                .lines()
                .map(|line_str| line_str.trim())
                .enumerate()
                .filter(|(_, line_str)| line_str.starts_with("///"))
                .map(|(line_nb, line_str)| CodeLine {
                    file_id,
                    line_nb,
                    line: line_str.get(4..).unwrap_or_default().to_owned(),
                })
                .collect::<Vec<CodeLine>>()
        })
        .collect::<Vec<CodeLine>>();
    let mut idx = 0;
    let mut tests = String::new();
    let mut code = false;
    let mut rust = false;
    let mut current = String::new();
    let mut start_line = None;
    tests.push_str(
"//! This file is not meant for manual editing.\n//! This file is automatically generated by `build.rs` script.\n//! Any changes made will be discarded.\n");
    for CodeLine { file_id, line_nb, line } in source_code {
        if line.starts_with("```") {
            if code {
                if rust {
                    idx += 1;
                    dbg!(&current);
                    tests.push_str(&format!(
                        "
#[test]
fn auto_doctest_{idx}() {{
    // Auto generated from {}:{}
{current}}}
",
                        files.get(file_id).unwrap().to_str().unwrap(),
                        start_line.unwrap()
                    ));
                };
                code = false;
                rust = false;
            } else {
                if matches!(line.as_str(), "```" | "```rust") {
                    start_line = Some(line_nb);
                    rust = true;
                } else {
                    rust = false;
                }
                code = true;
            }
            current.clear();
        } else if code && rust && !line.trim().is_empty() {
            dbg!(&line);
            current.push_str("    ");
            current.push_str(&line);
            current.push('\n');
        }
    }
    fs::write("tests/doctests.rs", tests).unwrap();
}
