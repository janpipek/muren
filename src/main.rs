use std::fs::rename;
use std::{env, path::Path, path::PathBuf};

use clap::{arg, command, value_parser, Arg, ArgAction, ArgMatches, Command};
use colored::Colorize;

extern crate unidecode;
use unidecode::unidecode;

struct RenameIntent {
    path: PathBuf,
    new_name: PathBuf,
}

enum RenameCommand {
    SetExt(String),
    Remove(String),
    Normalize,
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

fn suggest_renames(files: Vec<PathBuf>, command: RenameCommand) -> Vec<RenameIntent> {
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

fn extract_command(args_matches: &ArgMatches) -> Option<RenameCommand> {
    match args_matches.subcommand() {
        Some(("set-ext", matches)) => Some(RenameCommand::SetExt(
            matches.get_one::<String>("extension").unwrap().clone(),
        )),
        Some(("remove", matches)) => Some(RenameCommand::Remove(
            matches.get_one::<String>("pattern").unwrap().clone(),
        )),
        Some(("normalize", _)) => Some(RenameCommand::Normalize),
        _ => None,
    }
}

fn process_command(command: RenameCommand, files: Vec<PathBuf>, dry: bool, confirm: bool) {
    let intents = suggest_renames(files, command);
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
            Command::new("normalize")
                .about("Convert names to reasonable ASCII.")
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

    match extract_command(&matches) {
        Some(command) => {
            let files: Vec<PathBuf> = matches
                .subcommand()
                .unwrap()
                .1
                .get_many::<PathBuf>("path")
                .unwrap()
                .cloned()
                .collect();
            let dry = matches.get_flag("dry");
            let confirm = matches.get_flag("yes");
            // let files = matches.get_many::<PathBuf>("path").unwrap().cloned().collect();
            process_command(command, files, dry, confirm);
        }
        None => {
            eprintln!("No command provided.");
        }
    }
}
