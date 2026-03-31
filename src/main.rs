use std::{env, path::PathBuf};

use clap::{Arg, ArgAction, ArgMatches, Command, arg, command, value_parser};

use muren::recipes::{
    ChangeCaseRecipe, FixExtensionRecipe, NormalizeRecipe, PrefixRecipe, RemoveRecipe,
    RenameRecipe, ReplaceRecipe, SetExtensionRecipe, UuidRecipe,
};
use muren::{Config, run};

/// Parse the command line arguments
fn parse_config(matches: &ArgMatches) -> Config {
    let command = extract_command(matches);
    let files_args = matches.subcommand().unwrap().1.get_many::<PathBuf>("path");
    let files: Vec<PathBuf> = match files_args {
        Some(args) => args.cloned().collect(),
        None => vec![],
    };
    Config {
        command,
        dry: matches.get_flag("dry"),
        files,
        auto_confirm: matches.get_flag("yes"),
        show_unchanged: matches.get_flag("unchanged"),
    }
}

/// Extract the command from a subset of command line arguments
fn extract_command(args_matches: &ArgMatches) -> Box<dyn RenameRecipe> {
    match args_matches.subcommand() {
        None => panic!("No command provided"),
        Some((m, matches)) => match m {
            "set-ext" => Box::new(SetExtensionRecipe {
                extension: matches.get_one::<String>("extension").unwrap().clone(),
            }),
            "remove" => Box::new(RemoveRecipe {
                pattern: matches.get_one::<String>("pattern").unwrap().clone(),
            }),
            "normalize" => Box::new(NormalizeRecipe),
            "fix-ext" => Box::new(FixExtensionRecipe {
                append: matches.get_flag("append"),
            }),
            "prefix" => Box::new(PrefixRecipe {
                prefix: matches.get_one::<String>("prefix").unwrap().clone(),
            }),
            "replace" => Box::new(ReplaceRecipe {
                pattern: matches.get_one::<String>("pattern").unwrap().clone(),
                replacement: matches.get_one::<String>("replacement").unwrap().clone(),
                is_regex: matches.get_flag("regex"),
            }),
            "change-case" => Box::new(ChangeCaseRecipe {
                upper: matches.get_flag("upper"),
            }),
            "uuid" => Box::new(UuidRecipe {
                version: matches
                    .get_one::<String>("version")
                    .unwrap_or(&UuidRecipe::DEFAULT_VERSION.to_string())
                    .parse::<u8>()
                    .expect("Could not parse UUID version"),
            }),
            _ => panic!("Unknown command"),
        },
    }
}

fn create_cli_command() -> Command {
    let path_arg = Arg::new("path")
        .action(ArgAction::Append)
        .value_parser(value_parser!(PathBuf));

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
                -u --unchanged ... "Show unchanged files"
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
                .arg(path_arg.clone()),
        )
        .subcommand(
            Command::new("prefix")
                .about("Prefix with string")
                .arg(
                    Arg::new("prefix")
                        .help("the prefix to prepend to the name")
                        .action(ArgAction::Set)
                        .value_parser(value_parser!(String))
                        .required(true),
                )
                .arg(path_arg.clone()),
        )
        .subcommand(
            Command::new("replace")
                .about("Replace parts of the name")
                .arg(
                    Arg::new("pattern")
                        .help("Pattern to match")
                        .action(ArgAction::Set)
                        .value_parser(value_parser!(String))
                        .required(true),
                )
                .arg(
                    Arg::new("replacement")
                        .help("Pattern to match")
                        .action(ArgAction::Set)
                        .value_parser(value_parser!(String))
                        .required(true),
                )
                .arg(
                    arg!(
                        -R --regex ... "The pattern is a regex"
                    )
                    .action(clap::ArgAction::SetTrue),
                )
                .arg(path_arg.clone()),
        )
        .subcommand(
            Command::new("normalize")
                .about("Convert names to reasonable ASCII.")
                .arg(path_arg.clone()),
        )
        .subcommand(
            Command::new("fix-ext")
                .about("Fix extension according to the file contents.")
                .arg(path_arg.clone())
                .arg(
                    arg!(
                        -a --append ... "Append instead of replacing."
                    )
                    .action(clap::ArgAction::SetTrue),
                ),
        )
        .subcommand(
            Command::new("remove")
                .about("Remove part of a name from all files.")
                .arg(
                    Arg::new("pattern")
                        .help("The string to remove.")
                        .action(ArgAction::Set)
                        .value_parser(value_parser!(String))
                        .required(true),
                )
                .arg(path_arg.clone()),
        )
        .subcommand(
            Command::new("change-case")
                .about("Change case of all files.")
                .arg(path_arg.clone())
                .arg(
                    arg!(
                        -u --upper ... "Upper case (default: false)."
                    )
                    .action(clap::ArgAction::SetTrue),
                ),
        )
        .subcommand(
            Command::new("uuid")
                .about("Assign each file a UUID name.")
                .arg(path_arg.clone())
                .arg(Arg::new("version").short('v').action(clap::ArgAction::Set)),
        )
}

fn main() {
    let cli_command = create_cli_command();
    let matches = cli_command.get_matches();
    let config = parse_config(&matches);
    run(&config);
}
