pub mod commands;
pub mod extensions;

use colored::Colorize;
use std::collections::HashSet;
use std::fs::rename;
use std::ops::Deref;
use std::path::{Path, PathBuf};

extern crate unidecode;
use crate::commands::{RenameCommand, RenameIntent};

pub struct Config {
    pub command: Box<dyn RenameCommand>,
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
    command: &dyn RenameCommand,
    files: &[PathBuf],
    dry: bool,
    auto_confirm: bool,
    show_unchanged: bool,
) {
    let intents = command.suggest_renames(files);

    if contains_duplicates(&intents) {
        print!("All target names are not unique!");
        print_intents(&intents, false);
        return;
    }

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

fn contains_duplicates(intents: &[RenameIntent]) -> bool {
    let new_names: Vec<PathBuf> = intents
        .iter()
        .map(|intent| intent.new_name.clone())
        .collect();
    let mut uniq = HashSet::new();
    !new_names.into_iter().all(move |x| uniq.insert(x))
}

pub fn run(config: &Config) {
    process_command(
        config.command.deref(),
        &config.files,
        config.dry,
        config.auto_confirm,
        config.show_unchanged,
    );
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_contains_duplicates() {
        let a_to_b = RenameIntent {
            old_name: PathBuf::from("a"),
            new_name: PathBuf::from("b"),
        };
        let b_to_d = RenameIntent {
            old_name: PathBuf::from("b"),
            new_name: PathBuf::from("d"),
        };
        let c_to_d = RenameIntent {
            old_name: PathBuf::from("c"),
            new_name: PathBuf::from("d"),
        };

        assert!(contains_duplicates(&vec![b_to_d, c_to_d.clone()]));
        assert!(!contains_duplicates(&vec![a_to_b, c_to_d]));
        assert!(!contains_duplicates(&Vec::new()));
    }
}
