use thiserror::Error;

#[derive(Error, Debug)]
pub enum LicensError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Unsupported license: '{0}'. Run 'licens list' to see supported licenses.")]
    UnsupportedLicense(String),

}

pub type Result<T> = std::result::Result<T, LicensError>;
