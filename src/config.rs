use chrono::Datelike;
use std::env;
use std::fs;
use std::path::Path;

#[derive(Debug, Clone)]
pub struct AuthorInfo {
    pub name: String,
    pub email: Option<String>,
    pub year: i32,
}

impl AuthorInfo {
    pub fn resolve(
        name_override: Option<String>,
        email_override: Option<String>,
        year_override: Option<i32>,
    ) -> Self {
        let current_year = chrono::Utc::now().year();
        let year = year_override.unwrap_or(current_year);

        let mut name = name_override;
        let mut email = email_override;

        if name.is_none() || email.is_none() {
            if let Some((git_name, git_email)) = Self::read_git_config() {
                if name.is_none() {
                    name = git_name;
                }
                if email.is_none() {
                    email = git_email;
                }
            }
        }

        if name.is_none() || email.is_none() {
            if let Some((pkg_name, pkg_email)) = Self::read_package_json() {
                if name.is_none() {
                    name = pkg_name;
                }
                if email.is_none() {
                    email = pkg_email;
                }
            }
        }

        if name.is_none() {
            name = env::var("GIT_AUTHOR_NAME")
                .or_else(|_| env::var("USER"))
                .or_else(|_| env::var("USERNAME"))
                .ok();
        }

        if email.is_none() {
            email = env::var("GIT_AUTHOR_EMAIL").ok();
        }

        Self {
            name: name.unwrap_or_else(|| "<author>".to_string()),
            email,
            year,
        }
    }

    fn read_git_config() -> Option<(Option<String>, Option<String>)> {
        let mut git_config_path = Path::new(".git").join("config");
        if !git_config_path.exists() {
            if let Ok(home) = env::var("USERPROFILE").or_else(|_| env::var("HOME")) {
                git_config_path = Path::new(&home).join(".gitconfig");
            }
        }

        if let Ok(content) = fs::read_to_string(git_config_path) {
            let mut name = None;
            let mut email = None;
            let mut in_user_section = false;

            for line in content.lines() {
                let trimmed = line.trim();
                if trimmed.starts_with('[') && trimmed.ends_with(']') {
                    in_user_section = trimmed.eq_ignore_ascii_case("[user]");
                    continue;
                }

                if in_user_section {
                    if let Some((k, v)) = trimmed.split_once('=') {
                        let key = k.trim().to_lowercase();
                        let val = v.trim().trim_matches('"').trim_matches('\'').to_string();
                        if key == "name" && name.is_none() {
                            name = Some(val);
                        } else if key == "email" && email.is_none() {
                            email = Some(val);
                        }
                    }
                }
            }

            if name.is_some() || email.is_some() {
                return Some((name, email));
            }
        }
        None
    }

    fn read_package_json() -> Option<(Option<String>, Option<String>)> {
        let path = Path::new("package.json");
        if !path.exists() {
            return None;
        }

        let content = fs::read_to_string(path).ok()?;
        let json: serde_json::Value = serde_json::from_str(&content).ok()?;

        if let Some(author) = json.get("author") {
            if let Some(author_str) = author.as_str() {
                return Some(Self::parse_author_string(author_str));
            } else if let Some(author_obj) = author.as_object() {
                let name = author_obj.get("name").and_then(|v| v.as_str()).map(|s| s.to_string());
                let email = author_obj.get("email").and_then(|v| v.as_str()).map(|s| s.to_string());
                return Some((name, email));
            }
        }

        None
    }

    fn parse_author_string(input: &str) -> (Option<String>, Option<String>) {
        if let Some(start) = input.find('<') {
            if let Some(end) = input.find('>') {
                let name = input[..start].trim().to_string();
                let email = input[start + 1..end].trim().to_string();
                return (
                    if name.is_empty() { None } else { Some(name) },
                    if email.is_empty() { None } else { Some(email) },
                );
            }
        }
        (Some(input.trim().to_string()), None)
    }
}
