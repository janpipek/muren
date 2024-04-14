use std::{env, path::PathBuf};

use clap::{arg, command, value_parser, Arg, ArgAction, ArgMatches, Command};

use muren::{run, Config, RenameCommand};

fn parse_config(matches: &ArgMatches) -> Config {
    let command = extract_command(matches).unwrap();
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
    Config {
        command,
        dry,
        files,
        confirm,
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

fn create_cli_command() -> Command {
    command!()
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
        )
}

fn main() {
    let command = create_cli_command();
    let matches = command.get_matches();
    let config = parse_config(&matches);
    run(&config);
}
