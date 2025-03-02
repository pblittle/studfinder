use thiserror::Error;

/// Errors that can occur in the StudFinder application
/// 
/// This enum provides specific error variants for different failure modes
/// throughout the application, with context-rich information to aid in
/// troubleshooting and error handling.
#[derive(Error, Debug)]
pub enum StudFinderError {
    /// Error occurred during a database operation
    #[error("Database error during {operation}: {source}")]
    Database {
        /// The database operation that was being performed
        operation: String,
        /// The source error from the database layer
        source: Box<dyn std::error::Error + Send + Sync>,
    },

    /// Error occurred during a database migration
    #[error("Database migration to version {version} failed during {operation}: {source}")]
    Migration {
        /// The target schema version of the migration
        version: i32,
        /// The specific operation within the migration that failed
        operation: String,
        /// The source error from the database layer
        source: Box<dyn std::error::Error + Send + Sync>,
    },

    /// Failed to acquire the database lock
    #[error("Failed to acquire database lock during {operation}")]
    DatabaseLockFailed {
        /// The operation that was attempting to acquire the lock
        operation: String,
    },

    /// Database reset operation failed
    #[error("Database reset failed: {source}")]
    DatabaseResetFailed {
        /// The source error that caused the reset to fail
        source: Box<dyn std::error::Error + Send + Sync>,
    },

    /// Error occurred during image processing
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

impl From<anyhow::Error> for StudFinderError {
    fn from(err: anyhow::Error) -> Self {
        StudFinderError::Config(format!("Unexpected error: {}", err))
    }
}
