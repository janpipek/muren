use std::fs::rename;
use std::{env, path::Path, path::PathBuf};

use clap::{arg, command, value_parser, Arg, ArgAction, Command};
use colored::Colorize;
use file_format::FileFormat;

fn ensure_extension_many(files: Vec<PathBuf>, extension: &String, dry: bool) {
    for entry in files {
        ensure_extension_one(&entry, extension, dry);
    }
}

fn ensure_extension_one(path: &Path, extension: &String, dry: bool) {
    let mut new_name = PathBuf::from(path);
    new_name.set_extension(extension);
    maybe_rename(path, new_name.as_path(), dry);
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

fn remove_string_many(files: Vec<PathBuf>, pattern: &String, dry: bool) {
    for entry in files {
        remove_string_one(&entry, pattern, dry);
    }
}

fn remove_string_one(path: &Path, pattern: &String, dry: bool) {
    if path.to_string_lossy().contains(pattern) {
        let new_name = path.to_string_lossy().replace(pattern, "");
        maybe_rename(path, Path::new(&new_name), dry);
    } else {
        println!("= {0}", path.to_string_lossy());
    }
}

fn fix_extension_many(files: Vec<PathBuf>, dry: bool) {
    for entry in files {
        fix_extension_one(&entry, dry);
    }
}

fn fix_extension_one(path: &Path, dry: bool) {
    match FileFormat::from_file(path) {
        Ok(fmt) => ensure_extension_one(path, &String::from(fmt.extension()), dry),
        Err(_) => (),
    }
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
        )
        .subcommand(
            Command::new("fix-ext").about("Fix extension").arg(
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
            );
        }
        Some(("fix-ext", matches)) => {
            fix_extension_many(
                matches
                    .get_many::<PathBuf>("path")
                    .unwrap()
                    .cloned()
                    .collect(),
                matches.get_flag("dry"),
            );
        }
        _ => (),
    }

    /* let mut args: Vec<String> = env::args().collect();
    args.remove(0); // The executable itself
    let is_dry = is_dry(&mut args);
    match args.pop() {
        Some(extension) => ensure_extension_many(&args, &extension, is_dry),
        None => {
            eprintln!("Usage: muren <*files> <extension>");
            std::process::exit(-1);
        }
    }*/
}
