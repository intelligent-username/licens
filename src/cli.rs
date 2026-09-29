use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(
    name = "licens",
    version,
    about = "Generate project licenses and audit dependency licenses in one pass",
    long_about = None
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,

    /// License template key (e.g. mit, apache-2.0) when used directly: 'licens mit'
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
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Audit dependency licenses in the current project
    Audit {
        /// Output audit results format: 'markdown' or 'json'
        #[arg(short, long, default_value = "markdown")]
        format: String,

        /// Include dev dependencies in audit
        #[arg(long)]
        dev: bool,

        /// Include build dependencies in audit
        #[arg(long)]
        build: bool,
    },

    /// List all supported license templates
    List,

    /// Generate a license file for the project
    Generate {
        /// License identifier (e.g. mit, apache-2.0, gpl-3.0)
        license: String,
    },

    /// Direct license name invocation (e.g. 'licens mit')
    #[command(external_subcommand)]
    External(Vec<String>),
}
