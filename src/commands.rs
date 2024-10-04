use crate::extensions::{find_extensions_from_content, has_correct_extension};
use colored::Colorize;
use regex::Regex;
use std::fmt::{Display, Formatter};
use std::path::{Path, PathBuf};
use unidecode::unidecode;

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
            .map(|path| RenameIntent {
                old_name: path.clone(),
                new_name: self.suggest_new_name(path),
            })
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

    /// Compare whether old_names are converted to new expected_names using command.
    fn assert_renames_correctly(
        command: &dyn RenameCommand,
        old_names: &[&str],
        expected_names: &[&str],
    ) {
        let old: Vec<PathBuf> = old_names.iter().map(|&x| PathBuf::from(x)).collect();
        let new_intents = command.suggest_renames(&old);
        let new: Vec<PathBuf> = new_intents
            .iter()
            .map(|intent| intent.new_name.clone())
            .collect();
        let expected: Vec<PathBuf> = expected_names.iter().map(|&x| PathBuf::from(x)).collect();
        assert_eq!(expected, new);
    }

    #[test]
    fn test_set_prefix() {
        let p = Prefix {  prefix: String::from("a")  };
        let old_path = PathBuf::from("b");
        assert_eq!(p.suggest_new_name(&old_path), PathBuf::from("ab"))
    }

    mod test_replace {
        use super::*;

        #[test]
        fn test_regex() {
            // Regex really matching
            let replace = Replace {
                pattern: String::from("\\d"),
                replacement: String::from("a"),
                is_regex: true,
            };
            let old_path = PathBuf::from("a222");
            assert_eq!(replace.suggest_new_name(&old_path), PathBuf::from("aaaa"));

            // Regex present as literal
            let replace = Replace {
                pattern: String::from("a$"),
                replacement: String::from("a"),
                is_regex: true,
            };
            let old_path = PathBuf::from("a$a");
            assert_eq!(replace.suggest_new_name(&old_path), PathBuf::from("a$a"));
        }

        #[test]
        fn test_non_regex() {
            let command = Replace {
                pattern: String::from("a.c"),
                replacement: String::from("def"),
                is_regex: false,
            };
            let old_path = PathBuf::from("a.cabc");
            assert_eq!(command.suggest_new_name(&old_path), PathBuf::from("defabc"));
        }
    }

    mod test_change_case {
        use super::*;

        #[test]
        fn test_upper() {
            assert_renames_correctly(
                &ChangeCase { upper: true },
                &["Abc", "hnědý", "Αθήνα", "mountAIN🗻"],
                &["ABC", "HNĚDÝ", "ΑΘΉΝΑ", "MOUNTAIN🗻"]
            );
        }

        #[test]
        fn test_lower() {
            assert_renames_correctly(
                &ChangeCase { upper: false },
                &["Abc", "hnědý", "Αθήνα", "mountAIN🗻"],
                &["abc", "hnědý", "αθήνα", "mountain🗻"]
            );
        }
    }

    #[test]
    fn test_normalize() {
        assert_renames_correctly(
            &Normalize,
            &["Abc", "hnědý", "Αθήνα & Σπάρτη", "mountain🗻"],
            &["Abc", "hnedy", "Athena_&_Sparte", "mountain"]
        );
    }

    mod test_set_extension {
        use super::*;

        #[test]
        fn test_no_extension() {
            assert_renames_correctly(
                &SetExtension{ extension: String::from("") },
                &["a", "b", "c.jpg", ".gitignore"],
                &["a", "b", "c", ".gitignore"],
            );
        }

        #[test]
        fn test_some_extension() {
            assert_renames_correctly(
                &SetExtension{ extension: String::from("jpg") },
                &["a", "b", "c.jpg", ".gitignore"],
                &["a.jpg", "b.jpg", "c.jpg", ".gitignore.jpg"],
            );
        }
    }
}
