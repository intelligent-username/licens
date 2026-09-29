use clap::Parser;
use licens::audit::OutputFormat;
use licens::cli::{Cli, Commands};
use licens::commands::{run_audit, run_generate, run_list, AuditOptions, GenerateOptions};
use licens::error::Result;

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Some(Commands::List) => {
            run_list()?;
        }
        Some(Commands::Audit { format, dev, build }) => {
            let out_fmt = if format.eq_ignore_ascii_case("json") {
                OutputFormat::Json
            } else {
                OutputFormat::Markdown
            };

            run_audit(AuditOptions {
                format: out_fmt,
                include_dev: dev,
                include_build: build,
            })
            .await?;
        }
        Some(Commands::Generate { license }) => {
            run_generate(GenerateOptions {
                license_key: license,
                output_path: cli.output,
                author_name: cli.author,
                author_email: cli.email,
                year: cli.year,
            })?;
        }
        Some(Commands::External(args)) => {
            if let Some(license_key) = args.first() {
                run_generate(GenerateOptions {
                    license_key: license_key.clone(),
                    output_path: cli.output,
                    author_name: cli.author,
                    author_email: cli.email,
                    year: cli.year,
                })?;
            } else {
                run_list()?;
            }
        }
        None => {
            if let Some(license_key) = cli.license {
                run_generate(GenerateOptions {
                    license_key,
                    output_path: cli.output,
                    author_name: cli.author,
                    author_email: cli.email,
                    year: cli.year,
                })?;
            } else {
                run_list()?;
            }
        }
    }

    Ok(())
}
