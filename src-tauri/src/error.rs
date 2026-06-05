use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Database error: {0}")]
    Database(#[from] rusqlite::Error),

    #[error("JSON parse error: {0}")]
    JsonParse(#[from] serde_json::Error),

    #[error("Parse error: {message}")]
    Parse { message: String },

    #[error("Sync error ({origin}): {message}")]
    Sync { origin: String, message: String },

    #[error("Pricing error: {0}")]
    Pricing(String),

    #[error("Config error: {0}")]
    Config(String),

    #[error("Not found: {0}")]
    NotFound(String),
}

impl AppError {
    pub fn error_type(&self) -> &str {
        match self {
            AppError::Io(_) => "io_error",
            AppError::Database(_) => "database_error",
            AppError::JsonParse(_) => "json_parse_error",
            AppError::Parse { .. } => "parse_error",
            AppError::Sync { .. } => "sync_error",
            AppError::Pricing(_) => "pricing_error",
            AppError::Config(_) => "config_error",
            AppError::NotFound(_) => "not_found",
        }
    }

    /// Sanitized message — no file paths, usernames, or sensitive data
    pub fn sanitized_message(&self) -> String {
        match self {
            AppError::Io(e) => e.to_string(),
            AppError::Database(e) => e.to_string(),
            AppError::JsonParse(e) => e.to_string(),
            AppError::Parse { message } => message.clone(),
            AppError::Sync { origin, message } => format!("[{}] {}", origin, message),
            AppError::Pricing(msg) => msg.clone(),
            AppError::Config(msg) => msg.clone(),
            AppError::NotFound(msg) => msg.clone(),
        }
    }
}

impl From<AppError> for String {
    fn from(err: AppError) -> String {
        err.to_string()
    }
}

pub type AppResult<T> = Result<T, AppError>;
