use colored::Colorize;
use std::fs::rename;
use std::path::{Path, PathBuf};
use std::process;

extern crate unidecode;
use unidecode::unidecode;

pub struct RenameIntent {
    path: PathBuf,
    new_name: PathBuf,
}

pub enum RenameCommand {
    SetExtension(String),
    Remove(String),
    // TODO: Change to struct
    Prefix(String),
    FixExtension,
    Normalize,
}

pub struct Config {
    pub command: RenameCommand,
    pub dry: bool,
    pub files: Vec<PathBuf>,
    pub confirm: bool,
}

fn confirm_intents(intents: &Vec<RenameIntent>) -> bool {
    println!("The following files will be renamed:");
    print_intents(intents);
    println!("Do you want to continue? [y/N] ");
    let mut input = String::new();
    std::io::stdin().read_line(&mut input).unwrap();
    input.trim().to_lowercase() == "y"
}

fn print_intents(intents: &Vec<RenameIntent>) {
    for intent in intents {
        if intent.path == intent.new_name {
            continue;
        }
        println!(
            "- {0} → {1}",
            intent.path.to_string_lossy().red(),
            intent.new_name.to_string_lossy().green()
        );
    }
}

fn suggest_renames(files: &Vec<PathBuf>, command: &RenameCommand) -> Vec<RenameIntent> {
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
                let new_name = unidecode(&path_str).replace(" ", "_").to_lowercase();
                // let new_name = unidecode(path_str);

                RenameIntent {
                    path: path.clone(),
                    new_name: PathBuf::from(new_name),
                }
            }
            RenameCommand::FixExtension => {
                let possible_extensions = find_extensions_from_content(path);
                let new_name = {
                    if possible_extensions.is_empty()
                    {
                        path.clone()
                    } else {
                        let current_extension = path.extension();
                        if current_extension.is_none() {
                            let mut new_name = path.clone();
                            new_name.set_extension(&possible_extensions[0]);
                            new_name
                        }
                        else {
                            let extension = current_extension.unwrap().to_ascii_lowercase();
                            let extension_str = String::from(extension.to_string_lossy());
                            let is_correct_extension = possible_extensions.contains(&extension_str);
                            dbg!(extension_str);
                            if is_correct_extension {
                                path.clone() 
                            } else {
                                let mut new_name = path.clone();
                                new_name.set_extension(&possible_extensions[0]);
                                new_name
                            }
                        }
                    }
                };
                RenameIntent {
                    path: path.clone(),
                    new_name,
                }
            }
        })
        .collect()
}

fn infer_mimetype(path: &Path) -> Option<String> {
    let mut cmd = process::Command::new("file");
    let output = cmd.arg(path).arg("--brief").arg("--mime-type").output();
    match output {
        Ok(output) => {
            let output_str = String::from_utf8(output.stdout).unwrap();
            let mime_type = match output_str.strip_suffix("\n") {
                Some(s) => String::from(s),
                None => output_str
            };
            Some(mime_type)
        },
        Err(_) => None
    }
}

fn find_extensions_from_content(path: &Path) -> Vec<String> {
    match infer_mimetype(path) {
        None => vec![],
        Some(mime_type) => {
            let mime_type_str = mime_type.as_str();
            dbg!(&mime_type);
            match mime_type_str {
                "application/pdf" => vec![String::from("pdf")],
                "image/jpeg" => vec![String::from("jpeg"), String::from("jpg")],
                "text/csv" => vec![String::from("csv")],
                _other => vec![]
            }
        }
    }
}

fn maybe_rename(path: &Path, new_name: &Path, dry: bool) {
    if path == new_name {
        return;
    }
    if dry {
        println!(
            "- {0} → {1}",
            path.to_string_lossy().red(),
            new_name.to_string_lossy().green()
        )
    } else {
        match rename(path, new_name) {
            Ok(_) => println!(
                "{0} {1} → {2}",
                "✓".green(),
                path.to_string_lossy().red(),
                new_name.to_string_lossy().green()
            ),
            Err(_) => eprintln!(
                "{0} {1} → {2}",
                "✗".red(),
                path.to_string_lossy().red(),
                new_name.to_string_lossy().green()
            ),
        }
    }
}

fn process_command(command: &RenameCommand, files: &Vec<PathBuf>, dry: bool, confirm: bool) {
    let intents = suggest_renames(files, command);
    if dry {
        print_intents(&intents);
    } else if confirm || confirm_intents(&intents) {
        for intent in intents {
            maybe_rename(&intent.path, &intent.new_name, dry);
        }
    };
}

pub fn run(config: &Config) {
    process_command(&config.command, &config.files, config.dry, config.confirm);
}
