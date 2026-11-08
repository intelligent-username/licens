use clap::{Parser, Subcommand};
use std::io::IsTerminal;

pub(crate) fn use_color() -> bool {
    std::io::stdout().is_terminal()
        && std::env::var_os("NO_COLOR").is_none_or(|value| value.is_empty())
        && std::env::var("TERM").as_deref() != Ok("dumb")
}

pub(crate) fn paint(text: &str, style: &str, color: bool) -> String {
    if color {
        format!("\x1b[{style}m{text}\x1b[0m")
    } else {
        text.to_owned()
    }
}

pub(crate) fn table(
    headers: &[&str],
    rows: &[Vec<String>],
    styles: &[&str],
    color: bool,
) -> String {
    let widths: Vec<usize> = headers
        .iter()
        .enumerate()
        .map(|(column, header)| {
            rows.iter()
                .map(|row| row[column].chars().count())
                .max()
                .unwrap_or(0)
                .max(header.chars().count())
        })
        .collect();
    let border = |left: &str, middle: &str, right: &str| {
        paint(
            &format!(
                "{left}{}{right}\n",
                widths
                    .iter()
                    .map(|width| "─".repeat(width + 2))
                    .collect::<Vec<_>>()
                    .join(middle)
            ),
            "90",
            color,
        )
    };
    let row = |values: &[String], header: bool| {
        let mut line = paint("│", "90", color);
        for (column, value) in values.iter().enumerate() {
            let padded = format!("{value:<width$}", width = widths[column]);
            line.push(' ');
            line.push_str(&paint(
                &padded,
                if header { "1;37" } else { styles[column] },
                color,
            ));
            line.push(' ');
            line.push_str(&paint("│", "90", color));
        }
        line.push('\n');
        line
    };
    let mut output = border("╭", "┬", "╮");
    output.push_str(&row(
        &headers.iter().map(|s| (*s).to_owned()).collect::<Vec<_>>(),
        true,
    ));
    output.push_str(&border("├", "┼", "┤"));
    for values in rows {
        output.push_str(&row(values, false));
    }
    output.push_str(&border("╰", "┴", "╯"));
    output
}

fn help_section(title: &str, headers: &[&str], rows: &[(&str, &str)], color: bool) -> String {
    format!(
        "{}\n{}",
        paint(title, "1;36", color),
        table(
            headers,
            &rows
                .iter()
                .map(|(a, b)| vec![(*a).into(), (*b).into()])
                .collect::<Vec<_>>(),
            &["32", "0"],
            color
        )
    )
}

fn help_screen(command: &str) -> String {
    let color = use_color();
    let mut output = format!(
        "\n{} {}\n{}\n\nUsage: {{usage}}\n\n",
        paint("licens", "1;36", color),
        paint(env!("CARGO_PKG_VERSION"), "90", color),
        "A license file. One command. Offline."
    );
    if command.is_empty() {
        output.push_str(&help_section(
            "Commands",
            &["COMMAND", "WHAT IT DOES"],
            &[
                ("licens <license-or-alias>", "Create a license file"),
                (
                    "licens generate <license-or-alias>",
                    "Explicit generation syntax",
                ),
                ("licens list", "Browse licenses and your aliases"),
                ("licens assign m=mit", "Save a shortcut across projects"),
                (
                    "licens config key=value ...",
                    "Save your generation preferences",
                ),
                ("licens help <command>", "Read command-specific help"),
            ],
            color,
        ));
    }
    output.push_str(&help_section(
        "Options",
        &["OPTION", "WHAT IT DOES"],
        &[
            ("-o, --output <PATH>", "Output file (default: LICENSE)"),
            ("--author <NAME>", "Override the author name"),
            ("--email <EMAIL>", "Override the author email"),
            ("--year <YEAR>", "Override the current UTC year"),
            ("--force", "Replace an existing regular file without asking"),
            ("-h, --help, --h", "Show this help screen"),
        ],
        color,
    ));
    let examples: &[(&str, &str)] = match command {
        "assign" => &[
            ("licens assign m=mit", "Save the alias m for MIT"),
            ("licens m", "Use your saved shortcut"),
        ],
        "config" => &[
            (
                "licens config email=--ignore",
                "Never discover or include an email",
            ),
            (
                "licens config author=",
                "Clear the saved author; use discovery",
            ),
        ],
        "list" => &[("licens list", "Original keys are green; aliases are cyan")],
        _ => &[
            ("licens mit", "Create LICENSE in the current directory"),
            (
                "licens mit --author \"Your Name\"",
                "Use an explicit author",
            ),
        ],
    };
    output.push_str(&help_section(
        "Try it",
        &["EXAMPLE", "RESULT"],
        examples,
        color,
    ));
    if command.is_empty() {
        output.push_str("\n-V, --version  Show version\n");
    }
    output.push_str("\nSaved preferences apply across projects. Explicit flags take precedence.\n");
    output
}

#[derive(Parser, Debug)]
#[command(
    name = "licens",
    version,
    help_template = help_screen(""),
    disable_help_flag = true,
    about = "Add a license file to your project",
    long_about = None
)]
pub struct Cli {
    /// Print help
    #[arg(short = 'h', long = "help", alias = "h", action = clap::ArgAction::Help, global = true)]
    pub help: Option<bool>,

    #[command(subcommand)]
    pub command: Option<Commands>,

    /// License template key or saved alias (e.g. mit, apache-2.0)
    #[arg(index = 1)]
    pub license: Option<String>,

    /// Output path for the generated license file (defaults to LICENSE)
    #[arg(short, long, global = true)]
    pub output: Option<String>,

    /// Author name override for license generation
    #[arg(long, global = true)]
    pub author: Option<String>,

    /// Author email override
    #[arg(long, global = true)]
    pub email: Option<String>,

    /// Year override
    #[arg(long, global = true)]
    pub year: Option<i32>,

    /// Overwrite an existing regular file without asking
    #[arg(long, global = true)]
    pub force: bool,
}

#[cfg(test)]
mod tests {
    use super::Cli;
    use clap::{error::ErrorKind, Parser};

    #[test]
    fn normal_commands_do_not_require_a_help_flag() {
        for args in [
            vec!["licens", "mit"],
            vec!["licens", "list"],
            vec!["licens", "assign", "m=mit"],
            vec!["licens", "config", "email=--ignore"],
            vec!["licens", "generate", "mit"],
        ] {
            assert!(Cli::try_parse_from(args).is_ok());
        }
    }

    #[test]
    fn all_help_flags_work_at_root_and_on_subcommands() {
        for flag in ["-h", "--help", "--h"] {
            for command in [
                None,
                Some("list"),
                Some("assign"),
                Some("config"),
                Some("generate"),
            ] {
                let mut args = vec!["licens"];
                if let Some(command) = command {
                    args.push(command);
                }
                args.push(flag);
                let error = Cli::try_parse_from(args).unwrap_err();
                assert_eq!(error.kind(), ErrorKind::DisplayHelp);
                assert!(error.to_string().contains("Usage:"));
            }
        }
    }
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Save a user-wide license alias (e.g. m=mit)
    #[command(help_template = help_screen("assign"))]
    Assign {
        /// Alias assignment in the form <alias>=<license>
        assignment: String,
    },

    /// Save user-wide generation preferences (e.g. author=Name email=--ignore)
    #[command(help_template = help_screen("config"))]
    Config {
        #[arg(required = true, num_args = 1..)]
        assignments: Vec<String>,
    },

    /// List all supported license templates
    #[command(help_template = help_screen("list"))]
    List,

    /// Generate a license file for the project
    #[command(help_template = help_screen("generate"))]
    Generate {
        /// License identifier or saved alias (e.g. mit, apache-2.0)
        license: String,
    },
}
