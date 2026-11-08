use std::fs::{self, OpenOptions};
use std::io::{self, BufRead, IsTerminal, Write};
use std::path::Path;

use crate::config::AuthorInfo;
use crate::error::{LicensError, Result};
use crate::preferences::{self, UserConfig};
use crate::templates::find_license;

pub struct GenerateOptions {
    pub license_key: String,
    pub output_path: Option<String>,
    pub author_name: Option<String>,
    pub author_email: Option<String>,
    pub year: Option<i32>,
    pub force: bool,
}

fn confirm_overwrite(input: &mut impl BufRead, output: &mut impl Write) -> io::Result<bool> {
    write!(
        output,
        "This file already exists. Would you like to overwrite (Y/N)? "
    )?;
    output.flush()?;
    let mut answer = String::new();
    input.read_line(&mut answer)?;
    Ok(matches!(
        answer.trim().to_ascii_lowercase().as_str(),
        "y" | "yes"
    ))
}

pub fn run(opts: GenerateOptions) -> Result<()> {
    let builtin = find_license(&opts.license_key);
    let snapshot = match preferences::load() {
        Ok(config) => config,
        Err(LicensError::Alias(_)) if builtin.is_some() => UserConfig::default(),
        Err(error) => return Err(error),
    };
    let template = match builtin {
        Some(template) => template,
        None => crate::aliases::find_alias_in(&snapshot.aliases, &opts.license_key)?
            .ok_or_else(|| LicensError::UnsupportedLicense(opts.license_key.clone()))?,
    };

    let author_info = AuthorInfo::resolve_with_preferences(
        opts.author_name,
        opts.author_email,
        opts.year,
        &snapshot.preferences,
    )?;
    let author_display = match (&author_info.name, &author_info.email) {
        (Some(name), Some(email)) => format!("{name} <{email}>"),
        (Some(name), None) => name.clone(),
        (None, Some(email)) => email.clone(),
        (None, None) => String::new(),
    };

    let content = template.render_optional(
        author_info.year,
        (!author_display.is_empty()).then_some(author_display.as_str()),
    );
    let dest_path = match opts.output_path {
        Some(value) if !value.trim().is_empty() => value,
        Some(_) => "LICENSE".to_owned(),
        None => snapshot
            .preferences
            .get("output")
            .map(|value| value.trim())
            .filter(|value| !value.is_empty() && *value != "--ignore")
            .unwrap_or("LICENSE")
            .to_owned(),
    };
    let path = Path::new(&dest_path);

    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() && !parent.exists() {
            fs::create_dir_all(parent)?;
        }
    }

    let mut file = match OpenOptions::new().write(true).create_new(true).open(path) {
        Ok(file) => file,
        Err(error) if error.kind() == io::ErrorKind::AlreadyExists => {
            let metadata = fs::symlink_metadata(path)?;
            if !metadata.file_type().is_file() {
                return Err(LicensError::InvalidDestination(dest_path));
            }
            if !opts.force {
                if !io::stdin().is_terminal() || !io::stderr().is_terminal() {
                    return Err(LicensError::ConfirmationRequired(dest_path));
                }
                if !confirm_overwrite(&mut io::stdin().lock(), &mut io::stderr().lock())? {
                    eprintln!("Cancelled; existing file was not changed.");
                    return Ok(());
                }
            }
            OpenOptions::new().write(true).truncate(true).open(path)?
        }
        Err(error) => return Err(error.into()),
    };
    file.write_all(content.as_bytes())?;
    if author_info.missing_author && template.needs_author() {
        eprintln!("Author parameter is missing in the license. Set Git user.name (git config --global user.name \"Your Name\"), set package.json's author or GIT_AUTHOR_NAME, or pass --author \"Your Name\" for future use. Otherwise, edit the created license file to replace <author>.");
    }
    let author_suffix = if author_display.is_empty() {
        String::new()
    } else {
        format!(" for {author_display}")
    };
    let year_suffix = author_info
        .year
        .map(|year| format!(" ({year})"))
        .unwrap_or_default();
    println!(
        "Successfully generated {} file at '{}'{}{}.",
        template.spdx_id, dest_path, author_suffix, year_suffix
    );

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::confirm_overwrite;

    #[test]
    fn confirmation_accepts_only_yes() {
        for (input, expected) in [
            ("Y\n", true),
            (" yes \n", true),
            ("n\n", false),
            ("\n", false),
            ("maybe\n", false),
            ("", false),
        ] {
            let mut output = Vec::new();
            assert_eq!(
                confirm_overwrite(&mut input.as_bytes(), &mut output).unwrap(),
                expected
            );
            assert!(String::from_utf8(output)
                .unwrap()
                .contains("overwrite (Y/N)?"));
        }
    }
}
