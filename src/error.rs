use thiserror::Error;

#[derive(Error, Debug)]
pub enum StudFinderError {
    #[error("Database error: {0}")]
    Database(#[from] rusqlite::Error),

    #[error("Image processing error: {0}")]
    Image(#[from] image::ImageError),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Invalid image dimensions: {width}x{height}, minimum required: {min_width}x{min_height}")]
    InvalidDimensions {
        width: u32,
        height: u32,
        min_width: u32,
        min_height: u32,
    },

    #[error("Unsupported image format: {0}")]
    UnsupportedFormat(String),

    #[error("Piece not found: {0}")]
    PieceNotFound(String),

    #[error("Invalid configuration: {0}")]
    Config(String),
    
    #[error("No pieces detected in image")]
    NoPiecesDetected,
    
    #[error("Color detection failed: {0}")]
    ColorDetectionFailed(String),
    
    #[error("Template matching failed: {0}")]
    TemplateMatchingFailed(String),
    
    #[error("Database initialization failed: {0}")]
    DatabaseInitFailed(String),
    
    #[error("Database lock error")]
    DatabaseLockError,
}

pub type Result<T> = std::result::Result<T, StudFinderError>;

impl From<serde_json::Error> for StudFinderError {
    fn from(err: serde_json::Error) -> Self {
        StudFinderError::Config(err.to_string())
    }
}

impl From<std::sync::PoisonError<std::sync::MutexGuard<'_, rusqlite::Connection>>> for StudFinderError {
    fn from(_: std::sync::PoisonError<std::sync::MutexGuard<'_, rusqlite::Connection>>) -> Self {
        StudFinderError::DatabaseLockError
    }
}
