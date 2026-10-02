use thiserror::Error;

#[derive(Error, Debug)]
pub enum LicensError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("TOML parse error: {0}")]
    TomlDe(#[from] toml::de::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Network error: {0}")]
    Http(#[from] reqwest::Error),

    #[error("Unsupported license: '{0}'. Run 'licens list' to see supported licenses.")]
    UnsupportedLicense(String),

    #[error("No supported package manifest found in current directory.")]
    ManifestNotFound,
}

pub type Result<T> = std::result::Result<T, LicensError>;
