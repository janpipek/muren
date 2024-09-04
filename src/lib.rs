use colored::Colorize;
use regex::Regex;
use std::fmt::{Display, Formatter, Result};
use std::fs::rename;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};
use std::process::{self, exit};

extern crate unidecode;
use unidecode::unidecode;

pub struct RenameIntent {
    path: PathBuf,
    new_name: PathBuf,
}

impl RenameIntent {
    fn is_changed(&self) -> bool {
        self.path != self.new_name
    }
}

impl Display for RenameIntent {
    fn fmt(&self, f: &mut Formatter) -> Result {
        if self.is_changed() {
            write!(
                f,
                "{0} → {1}",
                self.path.to_string_lossy().red(),
                self.new_name.to_string_lossy().green()
            )
        } else {
            write!(f, "{0} =", self.path.to_string_lossy(),)
        }
    }
}

pub enum RenameCommand {
    SetExtension(String),
    Remove(String),
    Prefix(String),
    FixExtension(bool),
    Normalize,
    Replace(String, String, bool),
    ChangeCase(bool),
}

pub struct Config {
    pub command: RenameCommand,
    pub dry: bool,
    pub files: Vec<PathBuf>,
    pub auto_confirm: bool,
    pub show_unchanged: bool,
}

fn confirm_intents(intents: &Vec<RenameIntent>) -> bool {
    println!("The following files will be renamed:");
    print_intents(intents, false);
    println!("Do you want to continue? [y/N] ");
    let mut input = String::new();
    std::io::stdin().read_line(&mut input).unwrap();
    input.trim().to_lowercase() == "y"
}

fn print_intents(intents: &Vec<RenameIntent>, show_unchanged: bool) {
    for intent in intents {
        if intent.is_changed() || show_unchanged {
            println!("{}", intent);
        }
    }
}

fn suggest_renames(files: &[PathBuf], command: &RenameCommand) -> Vec<RenameIntent> {
    files
        .iter()
        .map(|path| match &command {
            RenameCommand::SetExtension(extension) => {
                let mut new_name = path.clone();
                new_name.set_extension(extension);
                RenameIntent {
                    path: path.clone(),
                    new_name,
                }
            }
            RenameCommand::Remove(pattern) => {
                let new_name = path.to_string_lossy().replace(pattern, "");
                RenameIntent {
                    path: path.clone(),
                    new_name: PathBuf::from(new_name),
                }
            }
            RenameCommand::Prefix(prefix) => {
                let mut new_name = prefix.clone();
                new_name.push_str(path.to_string_lossy().to_string().as_str());
                RenameIntent {
                    path: path.clone(),
                    new_name: PathBuf::from(new_name),
                }
            }
            RenameCommand::Normalize => {
                let path_str = path.to_string_lossy().to_string();
                let new_name = unidecode(&path_str).replace(' ', "_"); //#.to_lowercase();

                RenameIntent {
                    path: path.clone(),
                    new_name: PathBuf::from(new_name),
                }
            }
            RenameCommand::FixExtension(append) => {
                let possible_extensions = find_extensions_from_content(path);
                let new_name = if has_correct_extension(path, &possible_extensions) {
                    path.clone()
                } else {
                    let mut new_name = path.clone();
                    let mut new_extension = possible_extensions[0].clone();
                    if *append {
                        let old_extension = new_name.extension();
                        if old_extension.is_some() {
                            new_extension.insert(0, '.');
                            new_extension.insert_str(0, old_extension.unwrap().to_str().unwrap())
                        }
                    }
                    new_name.set_extension(new_extension);
                    new_name
                };
                RenameIntent {
                    path: path.clone(),
                    new_name,
                }
            }
            RenameCommand::Replace(pattern, replacement, is_regex) => {
                let path_str = path.to_string_lossy().to_string();
                let new_name = if *is_regex {
                    let re = Regex::new(pattern).unwrap();
                    re.replace_all(&path_str, replacement).to_string()
                } else {
                    path_str.replace(pattern, replacement)
                };
                RenameIntent {
                    path: path.clone(),
                    new_name: PathBuf::from(new_name),
                }
            }
            RenameCommand::ChangeCase(upper) => {
                let path_str = path.to_string_lossy().to_string();
                let new_name = match upper {
                    true => path_str.to_uppercase(),
                    false => path_str.to_lowercase(),
                };
                RenameIntent {
                    path: path.clone(),
                    new_name: PathBuf::from(new_name),
                }
            }
        })
        .collect()
}

fn infer_mimetype(path: &Path, mime_type: bool) -> Option<String> {
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

fn find_extensions_from_content(path: &Path) -> Vec<String> {
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

fn has_correct_extension(path: &Path, possible_extensions: &[String]) -> bool {
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

fn try_rename(path: &Path, new_name: &Path) -> bool {
    match rename(path, new_name) {
        Ok(_) => {
            println!(
                "{0} {1} → {2}",
                "✓".green(),
                path.to_string_lossy().red(),
                new_name.to_string_lossy().green()
            );
            true
        }
        Err(_) => {
            eprintln!(
                "{0} {1} → {2}",
                "✗".red(),
                path.to_string_lossy().red(),
                new_name.to_string_lossy().green()
            );
            false
        }
    }
}

fn process_command(
    command: &RenameCommand,
    files: &[PathBuf],
    dry: bool,
    auto_confirm: bool,
    show_unchanged: bool,
) {
    let intents = suggest_renames(files, command);
    if dry {
        print_intents(&intents, show_unchanged);
    } else {
        let confirmed = auto_confirm || {
            let changed_count = intents.iter().filter(|i| i.is_changed()).count();
            (changed_count == 0) || confirm_intents(&intents)
        };

        let mut renamed_count = 0;
        if confirmed {
            for intent in intents {
                if intent.is_changed() {
                    let renamed = try_rename(&intent.path, &intent.new_name);
                    renamed_count += renamed as i32;
                }
                if show_unchanged {
                    println!("{}", intent)
                }
            }
            println!("{renamed_count} files renamed.");
        }
    };
}

pub fn run(config: &Config) {
    process_command(
        &config.command,
        &config.files,
        config.dry,
        config.auto_confirm,
        config.show_unchanged,
    );
}
