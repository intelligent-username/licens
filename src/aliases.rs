use std::collections::BTreeMap;

use crate::error::{LicensError, Result};
use crate::preferences;
use crate::templates::{find_license, LicenseTemplate};

pub fn find_alias(query: &str) -> Result<Option<&'static LicenseTemplate>> {
    let config = preferences::load()?;
    find_alias_in(&config.aliases, query)
}

pub fn find_alias_in(
    aliases: &BTreeMap<String, String>,
    query: &str,
) -> Result<Option<&'static LicenseTemplate>> {
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
            "list" | "generate" | "assign" | "config" | "help" | "version"
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
    let mut config = preferences::load()?;
    config.aliases.insert(name.clone(), template.key.to_owned());
    preferences::save(&config)?;
    println!(
        "Assigned '{name}' to '{}'. Use 'licens {name}' in any project.",
        template.key
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn snapshot_lookup_normalizes_query_and_validates_target() {
        let aliases = BTreeMap::from([
            ("m".into(), "mit".into()),
            ("config".into(), "mit".into()),
            ("bad".into(), "unsupported".into()),
        ]);
        assert_eq!(find_alias_in(&aliases, " M ").unwrap().unwrap().key, "mit");
        assert_eq!(
            find_alias_in(&aliases, "config").unwrap().unwrap().key,
            "mit"
        );
        assert!(find_alias_in(&aliases, "missing").unwrap().is_none());
        let error = find_alias_in(&aliases, "bad").err().unwrap();
        assert!(matches!(error, LicensError::Alias(_)));
        assert!(error.to_string().contains("unsupported license"));
    }

    #[test]
    fn config_is_reserved_before_any_storage_access() {
        let error = assign(" CONFIG =mit").unwrap_err();
        assert!(matches!(error, LicensError::Alias(_)));
        assert!(error.to_string().contains("reserved"));
    }
}
