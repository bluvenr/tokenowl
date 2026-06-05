use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Database error: {0}")]
    Database(#[from] rusqlite::Error),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),

    #[error("CC Switch not detected")]
    CcSwitchNotDetected,

    #[error("CC Switch database locked")]
    CcSwitchDbLocked,

    #[error("CC Switch schema incompatible")]
    CcSwitchSchemaIncompatible,

    #[error("Sync error: {0}")]
    SyncError(String),

    #[error("Price calculation error: {0}")]
    PriceError(String),

    #[error("Configuration error: {0}")]
    ConfigError(String),

    #[error("Export error: {0}")]
    ExportError(String),

    #[error("Analysis error: {0}")]
    AnalysisError(String),
}

impl From<AppError> for String {
    fn from(err: AppError) -> Self {
        err.to_string()
    }
}

pub type AppResult<T> = Result<T, AppError>;
