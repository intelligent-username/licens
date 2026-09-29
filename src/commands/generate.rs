use std::fs;
use std::path::Path;

use crate::config::AuthorInfo;
use crate::error::{LicensError, Result};
use crate::templates::find_license;

pub struct GenerateOptions {
    pub license_key: String,
    pub output_path: Option<String>,
    pub author_name: Option<String>,
    pub author_email: Option<String>,
    pub year: Option<i32>,
}

pub fn run(opts: GenerateOptions) -> Result<()> {
    let template = find_license(&opts.license_key)
        .ok_or_else(|| LicensError::UnsupportedLicense(opts.license_key.clone()))?;

    let author_info = AuthorInfo::resolve(opts.author_name, opts.author_email, opts.year);
    let author_display = if let Some(ref email) = author_info.email {
        format!("{} <{}>", author_info.name, email)
    } else {
        author_info.name.clone()
    };

    let content = template.render(author_info.year, &author_display);
    let dest_path = opts.output_path.unwrap_or_else(|| "LICENSE".to_string());
    let path = Path::new(&dest_path);

    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() && !parent.exists() {
            fs::create_dir_all(parent)?;
        }
    }

    fs::write(path, content)?;
    println!(
        "Successfully generated {} file at '{}' for {} ({}).",
        template.spdx_id, dest_path, author_display, author_info.year
    );

    Ok(())
}
