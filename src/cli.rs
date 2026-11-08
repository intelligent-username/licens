use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(
    name = "licens",
    version,
    about = "Add a license file to your project",
    long_about = None
)]
pub struct Cli {
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

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Save a user-wide license alias (e.g. m=mit)
    Assign {
        /// Alias assignment in the form <alias>=<license>
        assignment: String,
    },

    /// Save user-wide generation preferences (e.g. author=Name email=--ignore)
    Config {
        #[arg(required = true, num_args = 1..)]
        assignments: Vec<String>,
    },

    /// List all supported license templates
    List,

    /// Generate a license file for the project
    Generate {
        /// License identifier or saved alias (e.g. mit, apache-2.0)
        license: String,
    },
}
