use chrono::Datelike;
use std::{collections::BTreeMap, env, fs, process::Command};

use crate::error::{LicensError, Result};

#[derive(Debug, Clone)]
pub struct AuthorInfo {
    pub name: String,
    pub email: Option<String>,
    pub year: i32,
    pub missing_author: bool,
}

#[derive(Debug, Clone)]
pub struct ResolvedAuthor {
    pub name: Option<String>,
    pub email: Option<String>,
    pub year: Option<i32>,
    pub missing_author: bool,
}

fn nonempty(value: Option<String>) -> Option<String> {
    value.and_then(|s| {
        let s = s.trim();
        (!s.is_empty()).then(|| s.to_owned())
    })
}

impl AuthorInfo {
    pub fn resolve(
        name_override: Option<String>,
        email_override: Option<String>,
        year_override: Option<i32>,
    ) -> Self {
        let (name, email, missing_author) =
            Self::resolve_identity(name_override, email_override, false, false);
        Self {
            name: name.unwrap_or_else(|| "<author>".to_owned()),
            email,
            year: year_override.unwrap_or_else(|| chrono::Utc::now().year()),
            missing_author,
        }
    }

    pub fn resolve_with_preferences(
        name_override: Option<String>,
        email_override: Option<String>,
        year_override: Option<i32>,
        preferences: &BTreeMap<String, String>,
    ) -> Result<ResolvedAuthor> {
        let year = match year_override {
            Some(year) => Some(year),
            None => match preferences.get("year").map(|value| value.trim()) {
                Some("--ignore") => None,
                Some(value) if !value.is_empty() => Some(value.parse::<i32>().map_err(|_| {
                    LicensError::Config(
                        "Preference 'year' must be an i32 integer or '--ignore'.".into(),
                    )
                })?),
                _ => Some(chrono::Utc::now().year()),
            },
        };
        let identity = |explicit: Option<String>, key: &str| match explicit {
            Some(value) => (nonempty(Some(value)), false),
            None => match nonempty(preferences.get(key).cloned()) {
                Some(value) if value == "--ignore" => (None, true),
                value => (value, false),
            },
        };
        let (name, suppress_name) = identity(name_override, "author");
        let (email, suppress_email) = identity(email_override, "email");
        let (name, email, missing_author) =
            Self::resolve_identity(name, email, suppress_name, suppress_email);
        Ok(ResolvedAuthor {
            name,
            email,
            year,
            missing_author,
        })
    }

    fn resolve_identity(
        name: Option<String>,
        email: Option<String>,
        suppress_name: bool,
        suppress_email: bool,
    ) -> (Option<String>, Option<String>, bool) {
        let mut name = nonempty(name);
        let mut email = nonempty(email);
        if !suppress_name && name.is_none() {
            name = Self::git_value("user.name");
        }
        if !suppress_email && email.is_none() {
            email = Self::git_value("user.email");
        }
        if (!suppress_name && name.is_none()) || (!suppress_email && email.is_none()) {
            if let Some((pkg_name, pkg_email)) = Self::read_package_json() {
                if !suppress_name && name.is_none() {
                    name = nonempty(pkg_name);
                }
                if !suppress_email && email.is_none() {
                    email = nonempty(pkg_email);
                }
            }
        }
        if !suppress_name && name.is_none() {
            name = ["GIT_AUTHOR_NAME", "USER", "USERNAME"]
                .iter()
                .find_map(|key| nonempty(env::var(key).ok()));
        }
        if !suppress_email && email.is_none() {
            email = nonempty(env::var("GIT_AUTHOR_EMAIL").ok());
        }
        let missing_author = !suppress_name && name.is_none();
        if missing_author {
            name = Some("<author>".to_owned());
        }
        (name, email, missing_author)
    }

    fn git_value(key: &str) -> Option<String> {
        let output = Command::new("git")
            .args(["config", "--get", key])
            .output()
            .ok()?;
        if !output.status.success() {
            return None;
        }
        nonempty(String::from_utf8(output.stdout).ok())
    }

    fn read_package_json() -> Option<(Option<String>, Option<String>)> {
        let content = fs::read_to_string("package.json").ok()?;
        let json: serde_json::Value = serde_json::from_str(&content).ok()?;
        let author = json.get("author")?;
        if let Some(s) = author.as_str() {
            return Some(Self::parse_author_string(s));
        }
        Some((
            author
                .get("name")
                .and_then(|v| v.as_str())
                .map(str::to_owned),
            author
                .get("email")
                .and_then(|v| v.as_str())
                .map(str::to_owned),
        ))
    }

    fn parse_author_string(input: &str) -> (Option<String>, Option<String>) {
        if let Some(start) = input.find('<') {
            if let Some(relative_end) = input[start + 1..].find('>') {
                if !input[..start].contains('>') {
                    return (
                        nonempty(Some(input[..start].to_owned())),
                        nonempty(Some(input[start + 1..start + 1 + relative_end].to_owned())),
                    );
                }
            }
            return (None, None);
        }
        if input.contains('>') {
            return (None, None);
        }
        (nonempty(Some(input.to_owned())), None)
    }
}
