use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("Database error: {0}")]
    Database(#[from] rusqlite::Error),

    #[error("Parse error: {0}")]
    Parse(String),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Insufficient balance in wallet '{0}'")]
    InsufficientBalance(String),

    #[error("Insufficient quantity for holding '{0}'")]
    InsufficientQuantity(String),

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

pub type Result<T> = std::result::Result<T, AppError>;

// Allow using ? with String parse errors inside repos/services
impl From<String> for AppError {
    fn from(s: String) -> Self {
        AppError::Parse(s)
    }
}
