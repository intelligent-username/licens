use std::collections::BTreeMap;

use crate::cli::{paint, table, use_color};
use crate::error::Result;
use crate::preferences;
use crate::templates::{find_license, LICENSES};

fn render(aliases: &BTreeMap<String, String>, color: bool) -> String {
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
    let rows: Vec<Vec<String>> = LICENSES
        .iter()
        .zip(&aliases_by_license)
        .map(|(license, aliases)| {
            vec![
                license.key.into(),
                if aliases.is_empty() {
                    "—".into()
                } else {
                    aliases.clone()
                },
                license.spdx_id.into(),
                license.category.as_str().into(),
                license.name.into(),
            ]
        })
        .collect();
    let mut output = format!(
        "\n{}  {}\n{}\n\n",
        paint("licens", "1;36", color),
        paint("/ License catalog", "90", color),
        paint(
            &format!(
                "{} built-in licenses · {} saved aliases",
                LICENSES.len(),
                aliases.len()
            ),
            "90",
            color
        )
    );
    output.push_str(&table(
        &["ORIGINAL", "ALIASES", "SPDX ID", "CATEGORY", "LICENSE"],
        &rows,
        &["32", "36", "90", "0", "0"],
        color,
    ));
    output.push_str(&format!(
        "\n{} originals  /  {} aliases\n",
        paint("Green", "32", color),
        paint("Cyan", "36", color)
    ));
    output.push_str("\nCreate a license  →  licens mit\nSave a shortcut   →  licens assign m=mit\nUse your shortcut →  licens m\n");
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
    print!("{}", render(&aliases, use_color()));
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
        let mit = output
            .lines()
            .find(|row| row.starts_with("│ mit "))
            .unwrap();
        assert!(mit.contains("m, mine"));
        assert!(output
            .lines()
            .any(|row| row.starts_with("│ apache-2.0") && row.contains(" a ")));
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
