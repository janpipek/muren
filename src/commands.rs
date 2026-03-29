use crate::extensions::{find_extensions_from_content, has_correct_extension};
use colored::Colorize;
use regex::Regex;
use std::fmt::{Display, Formatter};
use std::path::{Path, PathBuf};
use unidecode::unidecode;
use uuid::Uuid;

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

// Remove any occurrences of the pattern in the file name
pub struct RemoveCommand {
    pub pattern: String,
}

impl RenameCommand for RemoveCommand {
    fn suggest_new_name(&self, old_name: &Path) -> PathBuf {
        let new_name = old_name.to_string_lossy().replace(&self.pattern, "");
        PathBuf::from(new_name)
    }
}

// Replace one pattern with another in the file name
pub struct ReplaceCommand {
    // Pattern to be replaced
    pub pattern: String,

    // What will be placed in place of the pattern
    pub replacement: String,

    // Whether the pattern is a regular expression
    pub is_regex: bool,
}

impl RenameCommand for ReplaceCommand {
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

pub struct ChangeCaseCommand {
    pub upper: bool,
}

impl RenameCommand for ChangeCaseCommand {
    fn suggest_new_name(&self, old_name: &Path) -> PathBuf {
        let path_str = old_name.to_string_lossy().to_string();
        let new_name = match self.upper {
            true => path_str.to_uppercase(),
            false => path_str.to_lowercase(),
        };
        PathBuf::from(new_name)
    }
}

pub struct FixExtensionCommand {
    pub append: bool,
}

impl RenameCommand for FixExtensionCommand {
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

pub struct PrefixCommand {
    pub prefix: String,
}

impl RenameCommand for PrefixCommand {
    fn suggest_new_name(&self, old_name: &Path) -> PathBuf {
        let mut new_name = self.prefix.clone();
        new_name.push_str(old_name.to_string_lossy().to_string().as_str());
        PathBuf::from(new_name)
    }
}

pub struct UuidCommand {
    pub version: u8,
}

impl RenameCommand for UuidCommand {
    fn suggest_new_name(&self, old_name: &Path) -> PathBuf {
        let uuid = match self.version {
            4 => Uuid::new_v4(),
            7 => Uuid::now_v7(),
            _ => panic!("Unsupported uuid version"),
        };
        let new_name = uuid.to_string();
        let mut new_path = PathBuf::from(new_name);
        new_path.set_extension(
            match old_name.extension() {
                Some(ext) => ext.to_string_lossy().to_string(),
                None => String::new(),
            }
            .as_str(),
        );
        new_path
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
    fn test_prefix() {
        assert_renames_correctly(
            &PrefixCommand {
                prefix: String::from("a"),
            },
            &["b", "a"],
            &["ab", "aa"],
        );
    }

    mod test_replace {
        use super::*;

        #[test]
        fn test_regex() {
            let command = ReplaceCommand {
                pattern: String::from("\\d"),
                replacement: String::from("a"),
                is_regex: true,
            };
            assert_renames_correctly(
                &command,
                &["222", "abc", "answer_is_42", "\\d2"],
                &["aaa", "abc", "answer_is_aa", "\\da"],
            );
        }

        #[test]
        fn test_non_regex() {
            let command = ReplaceCommand {
                pattern: String::from("a.c"),
                replacement: String::from("def"),
                is_regex: false,
            };
            assert_renames_correctly(&command, &["a.c", "abc", "ABC"], &["def", "abc", "ABC"]);
        }
    }

    mod test_change_case {
        use super::*;

        #[test]
        fn test_upper() {
            assert_renames_correctly(
                &ChangeCaseCommand { upper: true },
                &["Abc", "hnědý", "Αθήνα", "mountAIN🗻"],
                &["ABC", "HNĚDÝ", "ΑΘΉΝΑ", "MOUNTAIN🗻"],
            );
        }

        #[test]
        fn test_lower() {
            assert_renames_correctly(
                &ChangeCaseCommand { upper: false },
                &["Abc", "hnědý", "Αθήνα", "mountAIN🗻"],
                &["abc", "hnědý", "αθήνα", "mountain🗻"],
            );
        }
    }

    mod test_uuid {
        use super::*;
        #[test]
        fn test_v4_with_extension() {
            let command = UuidCommand { version: 4 };
            let old_name = "whatever.txt";
            let path = command.suggest_new_name(&PathBuf::from(old_name));
            let path_stem = path.file_stem().unwrap().to_str().unwrap();
            assert!(Uuid::parse_str(path_stem).is_ok());
            assert_eq!(path.extension().unwrap().to_str().unwrap(), "txt");
        }

        #[test]
        fn test_v7_no_extension() {
            let command = UuidCommand { version: 7 };
            let old_name = "whatever";
            let new_path = command.suggest_new_name(&PathBuf::from(old_name));
            assert!(Uuid::parse_str(new_path.to_str().unwrap()).is_ok());
        }
    }

    #[test]
    fn test_normalize() {
        assert_renames_correctly(
            &Normalize,
            &["Abc", "hnědý", "Αθήνα & Σπάρτη", "mountain🗻"],
            &["Abc", "hnedy", "Athena_&_Sparte", "mountain"],
        );
    }

    mod test_set_extension {
        use super::*;

        #[test]
        fn test_no_extension() {
            assert_renames_correctly(
                &SetExtension {
                    extension: String::from(""),
                },
                &["a", "b", "c.jpg", ".gitignore"],
                &["a", "b", "c", ".gitignore"],
            );
        }

        #[test]
        fn test_some_extension() {
            assert_renames_correctly(
                &SetExtension {
                    extension: String::from("jpg"),
                },
                &["a", "b", "c.jpg", ".gitignore"],
                &["a.jpg", "b.jpg", "c.jpg", ".gitignore.jpg"],
            );
        }
    }

    #[test]
    fn test_remove() {
        assert_renames_correctly(
            &RemoveCommand {
                pattern: String::from("ab"),
            },
            &[".gitignore", "babe", "abABab"],
            &[".gitignore", "be", "AB"],
        )
    }
}
