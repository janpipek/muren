use std::io::ErrorKind;
use std::path::Path;
use std::process::{self, exit};

fn infer_mimetype(path: &Path, mime_type: bool) -> Option<String> {
    // TODO: Do something on windows :see_no_evil:
    let mut cmd = process::Command::new("file");
    let cmd_with_args = cmd.arg(path).arg("--brief");
    let cmd_with_args = if mime_type {
        cmd_with_args.arg("--mime-type")
    } else {
        cmd_with_args
    };

    let output = cmd_with_args.output();
    match output {
        Ok(output) => {
            let output_str = String::from_utf8(output.stdout).unwrap();
            let mime_type = match output_str.strip_suffix('\n') {
                Some(s) => String::from(s),
                None => output_str,
            };
            Some(mime_type)
        }
        Err(e) => match e.kind() {
            ErrorKind::NotFound => {
                eprintln!("Error: `file` probably not installed");
                exit(-1);
            }
            _ => panic!("{e}"),
        },
    }
}

pub fn find_extensions_from_content(path: &Path) -> Vec<String> {
    let mime_type_based = match infer_mimetype(path, true) {
        None => vec![],
        Some(mime_type) => {
            let mime_type_str = mime_type.as_str();
            match mime_type_str {
                "application/pdf" => vec![String::from("pdf")],
                "image/jpeg" => vec![String::from("jpeg"), String::from("jpg")],
                "image/png" => vec![String::from("png")],
                "text/csv" => vec![String::from("csv")],
                "text/html" => vec![String::from("html"), String::from("htm")],
                "text/x-script.python" => vec![String::from("py"), String::from("pyw")],
                _other => vec![],
            }
        }
    };

    let mut description_based = match infer_mimetype(path, false) {
        None => vec![],
        Some(description) => {
            let description_str = description.as_str();
            match description_str {
                "Apache Parquet" => vec![String::from("parquet"), String::from("pq")],
                _other => vec![],
            }
        }
    };

    let mut extensions = mime_type_based.clone();
    extensions.append(&mut description_based);
    extensions
}

pub fn has_correct_extension(path: &Path, possible_extensions: &[String]) -> bool {
    if possible_extensions.is_empty() {
        true
    } else {
        let current_extension = path.extension();
        if current_extension.is_none() {
            false
        } else {
            let extension = current_extension.unwrap().to_ascii_lowercase();
            let extension_str = String::from(extension.to_string_lossy());
            possible_extensions.contains(&extension_str)
        }
    }
}
