use std::collections::BTreeMap;
use std::env;
use std::io::{self, IsTerminal};

use crate::error::Result;
use crate::preferences;
use crate::templates::{find_license, LICENSES};

fn cell(value: &str, width: usize, color: &str, use_color: bool) -> String {
    let padded = format!("{value:<width$}");
    if use_color {
        format!("{color}{padded}\x1b[0m")
    } else {
        padded
    }
}

fn render(aliases: &BTreeMap<String, String>, use_color: bool) -> String {
    let aliases_by_license: Vec<String> = LICENSES
        .iter()
        .map(|license| {
            aliases
                .iter()
                .filter(|(_, target)| {
                    find_license(target).is_some_and(|target| target.key == license.key)
                })
                .map(|(alias, _)| alias.as_str())
                .collect::<Vec<_>>()
                .join(", ")
        })
        .collect();
    let alias_width = aliases_by_license
        .iter()
        .map(String::len)
        .max()
        .unwrap_or(0)
        .max("ALIASES".len());
    let mut output = format!(
        "{} {} {:<15} {:<18} NAME\n{}\n",
        cell("ORIGINAL", 15, "\x1b[32m", use_color),
        cell("ALIASES", alias_width, "\x1b[36m", use_color),
        "SPDX ID",
        "CATEGORY",
        "-".repeat(71 + alias_width)
    );

    for (license, aliases) in LICENSES.iter().zip(&aliases_by_license) {
        let aliases = if aliases.is_empty() { "-" } else { aliases };
        output.push_str(&format!(
            "{} {} {:<15} {:<18} {}\n",
            cell(license.key, 15, "\x1b[32m", use_color),
            cell(aliases, alias_width, "\x1b[36m", use_color),
            license.spdx_id,
            license.category.as_str(),
            license.name
        ));
    }
    output.push_str(
        "\nUse an original key or alias: 'licens mit' or 'licens m' after 'licens assign m=mit'.\n",
    );
    output
}

pub fn run() -> Result<()> {
    let aliases = match preferences::load() {
        Ok(config) => config.aliases,
        Err(error) => {
            eprintln!("Warning: cannot display saved aliases: {error}");
            BTreeMap::new()
        }
    };
    let use_color = io::stdout().is_terminal()
        && env::var_os("NO_COLOR").is_none_or(|value| value.is_empty())
        && env::var("TERM").as_deref() != Ok("dumb");
    print!("{}", render(&aliases, use_color));
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::render;
    use std::collections::BTreeMap;

    #[test]
    fn originals_and_aliases_use_distinct_colors() {
        let aliases = BTreeMap::from([("m".to_owned(), "mit".to_owned())]);
        let output = render(&aliases, true);
        let row = output
            .lines()
            .find(|row| row.contains("MIT License"))
            .unwrap();
        assert!(row.contains("\x1b[32mmit"));
        assert!(row.contains("\x1b[36mm"));
    }

    #[test]
    fn plain_table_groups_aliases_under_their_original() {
        let aliases = BTreeMap::from([
            ("m".to_owned(), "MIT".to_owned()),
            ("mine".to_owned(), "mit".to_owned()),
            ("a".to_owned(), "apache-2.0".to_owned()),
        ]);
        let output = render(&aliases, false);
        let mit = output.lines().find(|row| row.starts_with("mit ")).unwrap();
        assert!(mit.contains("m, mine"));
        assert!(output
            .lines()
            .any(|row| row.starts_with("apache-2.0") && row.contains(" a ")));
        assert!(!output.contains('\x1b'));
        assert_eq!(
            output
                .lines()
                .filter(|row| row.contains(" Copyleft "))
                .count(),
            6
        );
    }
}
