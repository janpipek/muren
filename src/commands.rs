use std::fmt::{Display, Formatter};
use std::path::{Path, PathBuf};
use colored::Colorize;
use regex::Regex;
use unidecode::unidecode;
use crate::extensions::{find_extensions_from_content, has_correct_extension};

#[derive(Clone)]
pub struct RenameIntent {
    pub old_name: PathBuf,
    pub new_name: PathBuf,
}

impl RenameIntent {
    /// Is the new name different from the old one?
    pub fn is_changed(&self) -> bool {
        self.old_name != self.new_name
    }
}

impl Display for RenameIntent {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
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


pub trait RenameCommand {
    fn suggest_new_name(&self, old_name: &Path) -> PathBuf;

    fn suggest_renames(&self, files: &[PathBuf]) -> Vec<RenameIntent> {
        files
            .iter()
            .map(|path| RenameIntent { old_name: path.clone(), new_name: self.suggest_new_name(path) })
            .collect()
    }
}

pub struct Normalize;

impl RenameCommand for Normalize {
    fn suggest_new_name(&self, old_name: &Path) -> PathBuf {
        let path_str = old_name.to_string_lossy().to_string();
        let new_name = unidecode(&path_str).replace(' ', "_"); //#.to_lowercase();
        PathBuf::from(new_name)
    }
}

pub struct SetExtension {
    pub extension: String,
}

impl RenameCommand for SetExtension {
    fn suggest_new_name(&self, old_name: &Path) -> PathBuf {
        let mut new_name = old_name.to_path_buf();
        new_name.set_extension(&self.extension);
        new_name
    }
}

pub struct Remove {
    pub pattern: String,
}

impl RenameCommand for Remove {
    fn suggest_new_name(&self, old_name: &Path) -> PathBuf {
        let new_name = old_name.to_string_lossy().replace(&self.pattern, "");
        PathBuf::from(new_name)
    }
}

pub struct Replace {
    pub pattern: String,
    pub replacement: String,
    pub is_regex: bool,
}

impl RenameCommand for Replace {
    fn suggest_new_name(&self, old_name: &Path) -> PathBuf {
        let path_str = old_name.to_string_lossy().to_string();
        let new_name = if self.is_regex {
            let re = Regex::new(&self.pattern).unwrap();
            re.replace_all(&path_str, &self.replacement).to_string()
        } else {
            path_str.replace(&self.pattern, &self.replacement)
        };
        PathBuf::from(new_name)
    }
}

pub struct ChangeCase {
    pub upper: bool,
}

impl RenameCommand for ChangeCase {
    fn suggest_new_name(&self, old_name: &Path) -> PathBuf {
        let path_str = old_name.to_string_lossy().to_string();
        let new_name = match self.upper {
            true => path_str.to_uppercase(),
            false => path_str.to_lowercase(),
        };
        PathBuf::from(new_name)
    }
}

pub struct FixExtension {
    pub append: bool,
}

impl RenameCommand for FixExtension {
    fn suggest_new_name(&self, old_name: &Path) -> PathBuf {
        let possible_extensions = find_extensions_from_content(old_name);
        let mut new_name = old_name.to_path_buf();
        if !has_correct_extension(old_name, &possible_extensions) {
            let mut new_extension = possible_extensions[0].clone();
            if self.append {
                if let Some(old_extension) = new_name.extension() {
                    new_extension.insert(0, '.');
                    new_extension.insert_str(0, old_extension.to_str().unwrap())
                }
            }
            new_name.set_extension(new_extension);
        };
        new_name
    }
}

pub struct Prefix {
    pub prefix: String,
}

impl RenameCommand for Prefix {
    fn suggest_new_name(&self, old_name: &Path) -> PathBuf {
        let mut new_name = self.prefix.clone();
        new_name.push_str(old_name.to_string_lossy().to_string().as_str());
        PathBuf::from(new_name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_set_prefix() {
        let p = Prefix { prefix: String::from("a") };
        let old_path = PathBuf::from("b");
        assert_eq!(
            p.suggest_new_name(&old_path),
            PathBuf::from("ab")
        )
    }
}