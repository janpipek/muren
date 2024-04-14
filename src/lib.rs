use colored::Colorize;
use std::fs::rename;
use std::path::{Path, PathBuf};

extern crate unidecode;
use unidecode::unidecode;

pub struct RenameIntent {
    path: PathBuf,
    new_name: PathBuf,
}

pub enum RenameCommand {
    SetExt(String),
    Remove(String),
    // TODO: Change to struct
    Prefix(String),
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
            RenameCommand::SetExt(extension) => {
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
        })
        .collect()
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
