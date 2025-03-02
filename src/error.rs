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
}

pub type Result<T> = std::result::Result<T, StudFinderError>;

impl From<serde_json::Error> for StudFinderError {
    fn from(err: serde_json::Error) -> Self {
        StudFinderError::Config(err.to_string())
    }
}