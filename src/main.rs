use std::fs::rename;
use std::{env, path::Path, path::PathBuf};

use clap::{arg, command, value_parser, Arg, ArgAction, Command};
use colored::Colorize;

struct RenameIntent {
    path: PathBuf,
    new_name: PathBuf,
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

fn ensure_extension_many(files: Vec<PathBuf>, extension: &String, dry: bool, confirm: bool) {
    let intents: Vec<RenameIntent> = files
        .iter()
        .map(|path| {
            let mut new_name = path.clone();
            new_name.set_extension(extension);
            RenameIntent {
                path: path.clone(),
                new_name,
            }
        })
        .collect();
    if dry {
        println!("The following files would be renamed:");
        print_intents(&intents);
    } else if confirm || confirm_intents(&intents) {
        for intent in intents {
            maybe_rename(&intent.path, &intent.new_name, dry);
        }
    };
}

fn maybe_rename(path: &Path, new_name: &Path, dry: bool) {
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

fn remove_string_many(files: Vec<PathBuf>, pattern: &String, dry: bool, confirm: bool) {
    let intents: Vec<RenameIntent> = files
        .iter()
        .map(|path| {
            let new_name = path.to_string_lossy().replace(pattern, "");
            RenameIntent {
                path: path.clone(),
                new_name: PathBuf::from(new_name),
            }
        })
        .collect();
    if dry {
        print_intents(&intents);
    } else if confirm || confirm_intents(&intents) {
        for intent in intents {
            maybe_rename(&intent.path, &intent.new_name, dry);
        }
    };
}

fn main() {
    let command = command!()
        .about("(mu)ltiple (ren)ames")
        .arg_required_else_help(true)
        .arg(
            arg!(
                -d --dry ... "Dry run"
            )
            .global(true)
            .action(clap::ArgAction::SetTrue),
        )
        .arg(
            arg!(
                -y --yes ... "Automatically confirm all actions"
            )
            .global(true)
            .action(clap::ArgAction::SetTrue),
        )
        .subcommand(
            Command::new("set-ext")
                .about("Change extension")
                .arg(
                    Arg::new("extension")
                        .help("Extension to set (dot excluding)")
                        .action(ArgAction::Set)
                        .value_parser(value_parser!(String))
                        .required(true),
                )
                .arg(
                    Arg::new("path")
                        .action(ArgAction::Append)
                        .value_parser(value_parser!(PathBuf)),
                ),
        )
        .subcommand(
            Command::new("remove")
                .about("Remove part of a name from all files.")
                .arg(
                    Arg::new("pattern")
                        .help("The string to remove")
                        .action(ArgAction::Set)
                        .value_parser(value_parser!(String))
                        .required(true),
                )
                .arg(
                    Arg::new("path")
                        .action(ArgAction::Append)
                        .value_parser(value_parser!(PathBuf)),
                ),
        );
    let matches = command.get_matches();

    match matches.subcommand() {
        Some(("set-ext", matches)) => {
            ensure_extension_many(
                matches
                    .get_many::<PathBuf>("path")
                    .unwrap()
                    .cloned()
                    .collect(),
                matches.get_one("extension").unwrap(),
                matches.get_flag("dry"),
                matches.get_flag("yes"),
            );
        }
        Some(("remove", matches)) => {
            remove_string_many(
                matches
                    .get_many::<PathBuf>("path")
                    .unwrap()
                    .cloned()
                    .collect(),
                matches.get_one("pattern").unwrap(),
                matches.get_flag("dry"),
                matches.get_flag("yes"),
            );
        }
        _ => (),
    }
}
