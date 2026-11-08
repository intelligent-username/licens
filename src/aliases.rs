use std::collections::BTreeMap;
use std::env;
use std::fs::{self, OpenOptions};
use std::io::{self, Write};
use std::path::{Path, PathBuf};

use crate::error::{LicensError, Result};
use crate::templates::{find_license, LicenseTemplate};

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
        .map(|path| path.join("licens/aliases.json"))
        .ok_or_else(|| {
            LicensError::Alias("Cannot locate your user configuration directory.".into())
        })
}

fn load(path: &Path) -> Result<BTreeMap<String, String>> {
    match fs::read(path) {
        Ok(bytes) => serde_json::from_slice(&bytes).map_err(|error| {
            LicensError::Alias(format!(
                "Cannot read aliases at '{}': {error}",
                path.display()
            ))
        }),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(BTreeMap::new()),
        Err(error) => Err(LicensError::Alias(format!(
            "Cannot read aliases at '{}': {error}",
            path.display()
        ))),
    }
}

pub fn find_alias(query: &str) -> Result<Option<&'static LicenseTemplate>> {
    let aliases = load(&config_path()?)?;
    match aliases.get(&query.trim().to_ascii_lowercase()) {
        Some(target) => find_license(target).map(Some).ok_or_else(|| {
            LicensError::Alias(format!(
                "Alias '{query}' refers to unsupported license '{target}'."
            ))
        }),
        None => Ok(None),
    }
}

pub fn assign(assignment: &str) -> Result<()> {
    let (name, target) = assignment.split_once('=').ok_or_else(|| {
        LicensError::Alias(
            "Use 'licens assign <alias>=<license>', for example 'licens assign m=mit'.".into(),
        )
    })?;
    let name = name.trim().to_ascii_lowercase();
    if find_license(&name).is_some()
        || matches!(
            name.as_str(),
            "list" | "generate" | "assign" | "help" | "version"
        )
    {
        return Err(LicensError::Alias(format!(
            "'{name}' is reserved and cannot be used as an alias."
        )));
    }
    if !name.starts_with(|c: char| c.is_ascii_alphabetic())
        || !name
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
    {
        return Err(LicensError::Alias("Alias names must start with a letter and contain only ASCII letters, digits, '-' or '_'.".into()));
    }
    let template = find_license(target)
        .ok_or_else(|| LicensError::UnsupportedLicense(target.trim().to_owned()))?;
    let path = config_path()?;
    let mut aliases = load(&path)?;
    aliases.insert(name.clone(), template.key.to_owned());
    let bytes = serde_json::to_vec_pretty(&aliases)
        .map_err(|error| LicensError::Alias(error.to_string()))?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let temporary = path.with_extension(format!("{}.tmp", std::process::id()));
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&temporary)?;
    let saved = (|| -> io::Result<()> {
        file.write_all(&bytes)?;
        file.write_all(b"\n")?;
        drop(file);
        fs::rename(&temporary, &path)
    })();
    if saved.is_err() {
        let _ = fs::remove_file(&temporary);
    }
    saved?;
    println!(
        "Assigned '{name}' to '{}'. Use 'licens {name}' in any project.",
        template.key
    );
    Ok(())
}
