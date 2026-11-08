use thiserror::Error;

#[derive(Error, Debug)]
pub enum LicensError {
    #[error("{0}")]
    Alias(String),

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Unsupported license: '{0}'. Run 'licens list' to see supported licenses.")]
    UnsupportedLicense(String),

    #[error("Cannot overwrite '{0}': destination is not a regular file.")]
    InvalidDestination(String),

    #[error("'{0}' already exists; use --force to overwrite without a terminal.")]
    ConfirmationRequired(String),
}

pub type Result<T> = std::result::Result<T, LicensError>;
