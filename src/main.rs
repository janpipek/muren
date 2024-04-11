use std::{env, path::PathBuf};
use std::fs::rename;


fn ensure_extension_many(files: &Vec<String>, extension: &String, dry: bool) {
    for entry in files {
        ensure_extension_one(PathBuf::from(entry), extension, dry);
    }
}


fn ensure_extension_one(path: PathBuf, extension: &String, dry: bool) {
    let mut new_name = path.clone();
    new_name.set_extension(extension);
    if dry {
        println!("Would rename {0} to {1}.", path.display(), new_name.display());
    }
    else {
        match rename(path.clone(), new_name.clone()) {
            Ok(_) => println!("Renamed {0} to {1}.", path.display(), new_name.display()),
            Err(_) => eprintln!("Failed to rename {0} to {1}.", path.display(), new_name.display()),
        }
    }
}


fn is_dry(args: &mut Vec<String>) -> bool {
    let index = args.iter().position(|x| *x == "--dry");
    match index {
        None => false,
        Some(_index) => {
            args.retain(|x| *x != "--dry");
            true
        }
    }
}


fn main() {
    let mut args: Vec<String> = env::args().collect();
    args.remove(0); // The executable itself
    let is_dry = is_dry(&mut args);
    match args.pop() {
        Some(extension) => ensure_extension_many(&args, &extension, is_dry),
        None => {
            eprintln!("Usage: muren <*files> <extension>");
            std::process::exit(-1);
        }
    }
}
