use std::{env, path::PathBuf, path::Path};
use std::fs::rename;

use colored::Colorize;
use clap::{arg, command, value_parser, Arg, ArgAction, Command};


fn ensure_extension_many(files: Vec<PathBuf>, extension: &String, dry: bool) {
    for entry in files {
        ensure_extension_one(&entry, extension, dry);
    }
}

fn ensure_extension_one(path: &PathBuf, extension: &String, dry: bool) {
        let mut new_name = path.clone();
        new_name.set_extension(extension);
        maybe_rename(path, new_name.as_path(), dry);
}

fn maybe_rename(path: &Path, new_name: &Path, dry: bool) {
    if dry {
        println!("- {0} → {1}", path.to_string_lossy().red(), new_name.to_string_lossy().green())
    }
    else {
        match rename(path, new_name) {
            Ok(_) => println!("{0} {1} → {2}", "✓".green(), path.to_string_lossy().red(), new_name.to_string_lossy().green()),
            Err(_) => eprintln!("{0} {1} → {2}", "✗".red(), path.to_string_lossy().red(), new_name.to_string_lossy().green()),
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
    }
    else {
        println!("= {0}", path.to_string_lossy());
    }
}


fn main() {
    let command = command!()
        .arg(arg!(
            -d --dry ... "Dry run"
        ).global(true).action(clap::ArgAction::SetTrue))
        .subcommand(
            Command::new("setext")
                .about("Change extension")
                .arg(
                    Arg::new("extension").help("Extension to set (dot excluding)")
                        .action(ArgAction::Set)
                        .value_parser(value_parser!(String))
                )                
                .arg(Arg::new("path").action(ArgAction::Append).value_parser(value_parser!(PathBuf)))

        )
        .subcommand(
            Command::new("remove")
                .about("Remove part of a name from all files.")
                .arg(
                    Arg::new("pattern").help("The string to remove").action(ArgAction::Set)
                    .value_parser(value_parser!(String))
                )
                .arg(Arg::new("path").action(ArgAction::Append).value_parser(value_parser!(PathBuf)))
        )
        ;
    let matches = command.get_matches();

    if let Some(matches) = matches.subcommand_matches("setext") {
        ensure_extension_many(
            matches.get_many::<PathBuf>("path").unwrap().cloned().collect(), 
            matches.get_one("extension").unwrap(), 
            matches.get_flag("dry")
        );
    }

    if let Some(matches) = matches.subcommand_matches("remove") {
        remove_string_many(
            matches.get_many::<PathBuf>("path").unwrap().cloned().collect(), 
            matches.get_one("pattern").unwrap(),
            matches.get_flag("dry")
        );
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
