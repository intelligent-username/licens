use clap::Parser;
use licens::cli::{Cli, Commands};
use licens::commands::{run_generate, run_list, GenerateOptions};
use licens::error::Result;

fn main() -> std::process::ExitCode {
    match run() {
        Ok(()) => std::process::ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("{error}");
            std::process::ExitCode::FAILURE
        }
    }
}

fn run() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Some(Commands::Assign { assignment }) => {
            licens::aliases::assign(&assignment)?;
        }
        Some(Commands::List) => {
            run_list()?;
        }
        Some(Commands::Generate { license }) => {
            run_generate(GenerateOptions {
                license_key: license,
                output_path: cli.output,
                author_name: cli.author,
                author_email: cli.email,
                year: cli.year,
                force: cli.force,
            })?;
        }
        None => {
            if let Some(license_key) = cli.license {
                run_generate(GenerateOptions {
                    license_key,
                    output_path: cli.output,
                    author_name: cli.author,
                    author_email: cli.email,
                    year: cli.year,
                    force: cli.force,
                })?;
            } else {
                run_list()?;
            }
        }
    }

    Ok(())
}
