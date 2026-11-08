use std::collections::BTreeMap;
use std::env;
use std::fs::{self, OpenOptions};
use std::io::{self, Write};
use std::path::{Path, PathBuf};

use serde_json::Value;

use crate::error::{LicensError, Result};

#[derive(Debug, Default, Clone)]
pub struct UserConfig {
    pub aliases: BTreeMap<String, String>,
    pub preferences: BTreeMap<String, String>,
}

fn config_path() -> Result<PathBuf> {
    #[cfg(windows)]
    let base = env::var_os("APPDATA").map(PathBuf::from);
    #[cfg(target_os = "macos")]
    let base =
        env::var_os("HOME").map(|home| PathBuf::from(home).join("Library/Application Support"));
    #[cfg(not(any(windows, target_os = "macos")))]
    let base = env::var_os("XDG_CONFIG_HOME")
        .filter(|value| !value.is_empty())
        .map(PathBuf::from)
        .or_else(|| env::var_os("HOME").map(|home| PathBuf::from(home).join(".config")));

    base.filter(|path| path.is_absolute())
        .map(|path| path.join("licens/config.json"))
        .ok_or_else(|| {
            LicensError::Config("Cannot locate your user configuration directory.".into())
        })
}

fn config_error(path: &Path, message: impl std::fmt::Display) -> LicensError {
    LicensError::Config(format!(
        "Cannot use configuration at '{}': {message}",
        path.display()
    ))
}

fn is_missing(path: &Path, error: &io::Error) -> bool {
    if error.kind() != io::ErrorKind::NotFound {
        return false;
    }
    // A missing child of a file or dangling symlink is not an absent configuration.
    for ancestor in path.ancestors() {
        match fs::symlink_metadata(ancestor) {
            Ok(_) => {
                return ancestor != path
                    && fs::metadata(ancestor).is_ok_and(|metadata| metadata.is_dir())
            }
            Err(error) if error.kind() == io::ErrorKind::NotFound => {}
            Err(_) => return false,
        }
    }
    false
}

fn validate_preference(key: &str, value: &str) -> std::result::Result<(), String> {
    if !matches!(key, "author" | "email" | "year" | "output") {
        return Err(format!("Unsupported preference key '{key}'."));
    }
    let value = value.trim();
    if key == "year" && !value.is_empty() && value != "--ignore" && value.parse::<i32>().is_err() {
        return Err("Preference 'year' must be an i32 integer or '--ignore'.".into());
    }
    Ok(())
}

fn section(value: &Value, key: &str, path: &Path) -> Result<BTreeMap<String, String>> {
    let Some(value) = value.get(key) else {
        return Ok(BTreeMap::new());
    };
    let object = value
        .as_object()
        .ok_or_else(|| config_error(path, format!("Section '{key}' must be an object.")))?;
    object
        .iter()
        .map(|(name, value)| {
            let value = value.as_str().ok_or_else(|| {
                config_error(path, format!("Section '{key}' must contain string values."))
            })?;
            Ok((name.clone(), value.to_owned()))
        })
        .collect()
}

fn load_at(path: &Path) -> Result<UserConfig> {
    let bytes = match fs::read(path) {
        Ok(bytes) => bytes,
        Err(error) if is_missing(path, &error) => {
            let legacy = path.with_file_name("aliases.json");
            let aliases = match fs::read(&legacy) {
                Ok(bytes) => serde_json::from_slice(&bytes).map_err(|error| {
                    LicensError::Alias(format!(
                        "Cannot read aliases at '{}': {error}",
                        legacy.display()
                    ))
                })?,
                Err(error) if is_missing(&legacy, &error) => BTreeMap::new(),
                Err(error) => {
                    return Err(LicensError::Alias(format!(
                        "Cannot read aliases at '{}': {error}",
                        legacy.display()
                    )))
                }
            };
            return Ok(UserConfig {
                aliases,
                preferences: BTreeMap::new(),
            });
        }
        Err(error) => return Err(config_error(path, error)),
    };
    let value: Value = serde_json::from_slice(&bytes).map_err(|error| config_error(path, error))?;
    let object = value
        .as_object()
        .ok_or_else(|| config_error(path, "The root must be an object."))?;
    for key in object.keys() {
        if !matches!(key.as_str(), "aliases" | "preferences") {
            return Err(config_error(path, format!("Unsupported section '{key}'.")));
        }
    }
    let config = UserConfig {
        aliases: section(&value, "aliases", path)?,
        preferences: section(&value, "preferences", path)?,
    };
    for (key, value) in &config.preferences {
        validate_preference(key, value).map_err(|error| config_error(path, error))?;
    }
    Ok(config)
}

pub fn load() -> Result<UserConfig> {
    load_at(&config_path()?)
}

fn save_at(config: &UserConfig, path: &Path) -> Result<()> {
    for (key, value) in &config.preferences {
        validate_preference(key, value).map_err(|error| config_error(path, error))?;
    }
    let string_map = |values: &BTreeMap<String, String>| {
        Value::Object(
            values
                .iter()
                .map(|(key, value)| (key.clone(), Value::String(value.clone())))
                .collect(),
        )
    };
    let mut object = serde_json::Map::new();
    object.insert("aliases".into(), string_map(&config.aliases));
    object.insert("preferences".into(), string_map(&config.preferences));
    let bytes = serde_json::to_vec_pretty(&Value::Object(object))
        .map_err(|error| config_error(path, error))?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|error| config_error(path, error))?;
    }
    let temporary = path.with_extension(format!("{}.tmp", std::process::id()));
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&temporary)
        .map_err(|error| config_error(path, error))?;
    let saved = (|| -> io::Result<()> {
        file.write_all(&bytes)?;
        file.write_all(b"\n")?;
        drop(file);
        fs::rename(&temporary, path)
    })();
    if saved.is_err() {
        let _ = fs::remove_file(&temporary);
    }
    saved.map_err(|error| config_error(path, error))
}

pub fn save(config: &UserConfig) -> Result<()> {
    save_at(config, &config_path()?)
}

fn parse_assignments(assignments: &[String]) -> Result<Vec<(String, String)>> {
    if assignments.is_empty() {
        return Err(LicensError::Config(
            "Use 'licens config key=value ...' with at least one assignment.".into(),
        ));
    }
    assignments
        .iter()
        .map(|assignment| {
            let (key, value) = assignment.split_once('=').ok_or_else(|| {
                LicensError::Config("Configuration assignments must use key=value.".into())
            })?;
            let (key, value) = (key.trim(), value.trim());
            validate_preference(key, value).map_err(LicensError::Config)?;
            Ok((key.to_owned(), value.to_owned()))
        })
        .collect()
}

fn configure_at(assignments: Vec<(String, String)>, path: &Path) -> Result<()> {
    let mut config = load_at(path)?;
    for (key, value) in assignments {
        if value.is_empty() {
            config.preferences.remove(&key);
        } else {
            config.preferences.insert(key, value);
        }
    }
    save_at(&config, path)
}

pub fn configure(assignments: &[String]) -> Result<()> {
    let assignments = parse_assignments(assignments)?;
    configure_at(assignments, &config_path()?)?;
    println!("Configuration updated.");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn loading_is_read_only_and_new_config_is_authoritative() {
        let root = tempfile::tempdir().unwrap();
        let path = root.path().join("licens/config.json");
        assert!(load_at(&path).unwrap().aliases.is_empty());
        assert!(!path.parent().unwrap().exists());
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        let legacy = path.with_file_name("aliases.json");
        let bytes = br#"{"m":"mit","config":"mit"}"#;
        fs::write(&legacy, bytes).unwrap();
        let imported = load_at(&path).unwrap();
        assert_eq!(imported.aliases["m"], "mit");
        assert_eq!(imported.aliases["config"], "mit");
        assert!(imported.preferences.is_empty());
        assert!(!path.exists());
        fs::write(&path, b"{}").unwrap();
        assert!(load_at(&path).unwrap().aliases.is_empty());
        assert_eq!(fs::read(legacy).unwrap(), bytes);
    }

    #[test]
    fn invalid_new_config_fails_closed_with_its_path() {
        let root = tempfile::tempdir().unwrap();
        let path = root.path().join("config.json");
        for bytes in [
            "not JSON",
            "[]",
            r#"{"extra":{}}"#,
            r#"{"aliases":null}"#,
            r#"{"aliases":{"m":1}}"#,
            r#"{"preferences":[]}"#,
            r#"{"preferences":{"author":false}}"#,
            r#"{"preferences":{"Author":"Ada"}}"#,
            r#"{"preferences":{"year":"2147483648"}}"#,
        ] {
            fs::write(&path, bytes).unwrap();
            let error = load_at(&path).unwrap_err();
            assert!(matches!(error, LicensError::Config(_)));
            assert!(error.to_string().contains(&path.display().to_string()));
            assert_eq!(fs::read(&path).unwrap(), bytes.as_bytes());
        }
        fs::remove_file(&path).unwrap();
        fs::create_dir(&path).unwrap();
        assert!(matches!(load_at(&path), Err(LicensError::Config(_))));
        let invalid_root = root.path().join("not-a-directory");
        fs::write(&invalid_root, b"keep").unwrap();
        assert!(matches!(
            load_at(&invalid_root.join("licens/config.json")),
            Err(LicensError::Config(_))
        ));
    }

    #[test]
    fn malformed_legacy_preserves_alias_error() {
        let root = tempfile::tempdir().unwrap();
        let path = root.path().join("config.json");
        let legacy = path.with_file_name("aliases.json");
        fs::write(&legacy, b"not JSON").unwrap();
        let error = load_at(&path).unwrap_err();
        assert!(matches!(error, LicensError::Alias(_)));
        assert!(error.to_string().contains("Cannot read aliases"));
        assert_eq!(fs::read(legacy).unwrap(), b"not JSON");
        assert!(!path.exists());
    }

    #[test]
    fn repeated_saves_replace_config_and_preserve_maps_and_legacy() {
        let root = tempfile::tempdir().unwrap();
        let path = root.path().join("config.json");
        let legacy = path.with_file_name("aliases.json");
        let bytes = b"{ \"m\": \"mit\" }\n";
        fs::write(&legacy, bytes).unwrap();
        let mut config = load_at(&path).unwrap();
        config.preferences.insert("email".into(), "--ignore".into());
        save_at(&config, &path).unwrap();
        config.aliases.insert("b".into(), "bsd-3-clause".into());
        save_at(&config, &path).unwrap();
        config
            .preferences
            .insert("year".into(), "-2147483648".into());
        save_at(&config, &path).unwrap();
        let saved = load_at(&path).unwrap();
        assert_eq!(saved.aliases, config.aliases);
        assert_eq!(saved.preferences, config.preferences);
        assert_eq!(fs::read(legacy).unwrap(), bytes);
        let value: Value = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
        assert_eq!(value.as_object().unwrap().len(), 2);
        assert_eq!(value["aliases"]["m"], "mit");
        assert_eq!(value["preferences"]["email"], "--ignore");
    }

    #[test]
    fn failed_saves_preserve_destination_and_clean_up_own_temporary_file() {
        let root = tempfile::tempdir().unwrap();
        let path = root.path().join("config.json");
        fs::write(&path, b"{}").unwrap();
        let mut config = UserConfig::default();
        config.preferences.insert("year".into(), "invalid".into());
        assert!(matches!(
            save_at(&config, &path),
            Err(LicensError::Config(_))
        ));
        assert_eq!(fs::read(&path).unwrap(), b"{}");
        let absent = root.path().join("absent/config.json");
        assert!(save_at(&config, &absent).is_err());
        assert!(!absent.parent().unwrap().exists());
        config.preferences.clear();
        let temporary = path.with_extension(format!("{}.tmp", std::process::id()));
        fs::write(&temporary, b"another writer").unwrap();
        assert!(save_at(&config, &path).is_err());
        assert_eq!(fs::read(&temporary).unwrap(), b"another writer");
        assert_eq!(fs::read(&path).unwrap(), b"{}");
        fs::remove_file(&temporary).unwrap();
        fs::remove_file(&path).unwrap();
        fs::create_dir(&path).unwrap();
        fs::write(path.join("keep"), b"keep").unwrap();
        assert!(save_at(&config, &path).is_err());
        assert_eq!(fs::read(path.join("keep")).unwrap(), b"keep");
        assert!(!temporary.exists());
    }

    fn configure_test(assignments: &[&str], path: &Path) -> Result<()> {
        let assignments = assignments
            .iter()
            .map(|value| (*value).to_owned())
            .collect::<Vec<_>>();
        configure_at(parse_assignments(&assignments)?, path)
    }

    #[test]
    fn configuration_batches_trim_clear_and_preserve_equals_and_order() {
        let root = tempfile::tempdir().unwrap();
        let path = root.path().join("config.json");
        configure_test(
            &[
                " author = Ada=Example ",
                "email=--ignore",
                "year=2147483647",
                "output=LICENSE.md",
            ],
            &path,
        )
        .unwrap();
        let config = load_at(&path).unwrap();
        assert_eq!(config.preferences["author"], "Ada=Example");
        assert_eq!(config.preferences["email"], "--ignore");
        assert_eq!(config.preferences["year"], "2147483647");
        configure_test(
            &["author=First", "author=Last", "email=  ", "year=--ignore"],
            &path,
        )
        .unwrap();
        let config = load_at(&path).unwrap();
        assert_eq!(config.preferences["author"], "Last");
        assert!(!config.preferences.contains_key("email"));
        assert_eq!(config.preferences["year"], "--ignore");
        assert_eq!(config.preferences["output"], "LICENSE.md");
        configure_test(&["author=", "year=", "output="], &path).unwrap();
        assert!(load_at(&path).unwrap().preferences.is_empty());
    }

    #[test]
    fn invalid_batches_do_not_create_or_change_config() {
        let root = tempfile::tempdir().unwrap();
        let existing = root.path().join("config.json");
        let absent = root.path().join("absent/config.json");
        fs::write(&existing, b"{}").unwrap();
        for assignments in [
            vec![],
            vec!["author=Ada", "missing-equals"],
            vec!["author=Ada", "unknown=value"],
            vec!["Author=Ada"],
            vec!["year=invalid", "year=2030"],
            vec!["year=2147483648"],
            vec!["year=-2147483649"],
        ] {
            assert!(configure_test(&assignments, &existing).is_err());
            assert!(configure_test(&assignments, &absent).is_err());
            assert_eq!(fs::read(&existing).unwrap(), b"{}");
            assert!(!absent.parent().unwrap().exists());
        }
        fs::write(&existing, b"not JSON").unwrap();
        assert!(configure_test(&["author=Ada"], &existing).is_err());
        assert_eq!(fs::read(existing).unwrap(), b"not JSON");
    }
}
