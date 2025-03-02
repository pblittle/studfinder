use thiserror::Error;

/// Errors that can occur in the StudFinder application
/// 
/// This enum provides specific error variants for different failure modes
/// throughout the application, with context-rich information to aid in
/// troubleshooting and error handling.
#[derive(Error, Debug)]
pub enum StudFinderError {
    #[error("Database error: {0}")]
    Database(#[from] rusqlite::Error),

    #[error("Image processing error: {0}")]
    Image(#[from] image::ImageError),

    /// I/O error occurred
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// Image dimensions are invalid for processing
    #[error("Invalid image dimensions: {width}x{height}, minimum required: {min_width}x{min_height}")]
    InvalidDimensions {
        /// The actual width of the image
        width: u32,
        /// The actual height of the image
        height: u32,
        /// The minimum width required
        min_width: u32,
        /// The minimum height required
        min_height: u32,
    },

    /// Image format is not supported
    #[error("Unsupported image format: {0}")]
    UnsupportedFormat(String),

    /// Requested piece was not found in the database
    #[error("Piece not found: {0}")]
    PieceNotFound(String),

    /// Configuration error occurred
    #[error("Invalid configuration: {0}")]
    Config(String),
    
    /// No LEGO pieces were detected in the processed image
    #[error("No pieces detected in image")]
    NoPiecesDetected,
    
    /// Color detection failed
    #[error("Color detection failed: {0}")]
    ColorDetectionFailed(String),
    
    /// Template matching failed
    #[error("Template matching failed: {0}")]
    TemplateMatchingFailed(String),
}

/// A specialized Result type for StudFinder operations
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
