mod extensions;

use colored::Colorize;
use regex::Regex;
use std::fmt::{Display, Formatter, Result};
use std::fs::rename;
use std::path::{Path, PathBuf};

use crate::extensions::{find_extensions_from_content, has_correct_extension};

extern crate unidecode;
use unidecode::unidecode;

pub struct RenameIntent {
    old_name: PathBuf,
    new_name: PathBuf,
}

impl RenameIntent {
    /// Is the new name different from the old one?
    fn is_changed(&self) -> bool {
        self.old_name != self.new_name
    }
}

impl Display for RenameIntent {
    fn fmt(&self, f: &mut Formatter) -> Result {
        if self.is_changed() {
            write!(
                f,
                "{0} → {1}",
                self.old_name.to_string_lossy().red(),
                self.new_name.to_string_lossy().green()
            )
        } else {
            write!(f, "{0} =", self.old_name.to_string_lossy(),)
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

/// Print all renames
fn print_intents(intents: &Vec<RenameIntent>, show_unchanged: bool) {
    for intent in intents {
        if intent.is_changed() || show_unchanged {
            println!("{}", intent);
        }
    }
}

/// Find a new name for a single file
fn suggest_rename(path: &PathBuf, command: &RenameCommand) -> RenameIntent {
    RenameIntent {
        old_name: path.clone(),
        new_name: match &command {
            RenameCommand::SetExtension(extension) => {
                let mut new_name = path.clone();
                new_name.set_extension(extension);
                new_name
            }
            RenameCommand::Remove(pattern) => {
                let new_name = path.to_string_lossy().replace(pattern, "");
                PathBuf::from(new_name)
            }
            RenameCommand::Prefix(prefix) => {
                let mut new_name = prefix.clone();
                new_name.push_str(path.to_string_lossy().to_string().as_str());
                PathBuf::from(new_name)
            }
            RenameCommand::Normalize => {
                let path_str = path.to_string_lossy().to_string();
                let new_name = unidecode(&path_str).replace(' ', "_"); //#.to_lowercase();
                PathBuf::from(new_name)
            }
            RenameCommand::FixExtension(append) => {
                let possible_extensions = find_extensions_from_content(path);
                let mut new_name = path.clone();
                if !has_correct_extension(path, &possible_extensions) {
                    let mut new_extension = possible_extensions[0].clone();
                    if *append {
                        let old_extension = new_name.extension();
                        if old_extension.is_some() {
                            new_extension.insert(0, '.');
                            new_extension.insert_str(0, old_extension.unwrap().to_str().unwrap())
                        }
                    }
                    new_name.set_extension(new_extension);
                };
                new_name
            }
            RenameCommand::Replace(pattern, replacement, is_regex) => {
                let path_str = path.to_string_lossy().to_string();
                let new_name = if *is_regex {
                    let re = Regex::new(pattern).unwrap();
                    re.replace_all(&path_str, replacement).to_string()
                } else {
                    path_str.replace(pattern, replacement)
                };
                PathBuf::from(new_name)
            }
            RenameCommand::ChangeCase(upper) => {
                let path_str = path.to_string_lossy().to_string();
                let new_name = match upper {
                    true => path_str.to_uppercase(),
                    false => path_str.to_lowercase(),
                };
                PathBuf::from(new_name)
            }
        },
    }
}

fn suggest_renames(files: &[PathBuf], command: &RenameCommand) -> Vec<RenameIntent> {
    files
        .iter()
        .map(|path| suggest_rename(path, command))
        .collect()
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
                    let renamed = try_rename(&intent.old_name, &intent.new_name);
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
