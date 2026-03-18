use thiserror::Error;

#[derive(Error, Debug)]
pub enum ThumbsError {
    #[error("File not found: {0}")]
    FileNotFound(String),

    #[error("Unsupported format: {0}")]
    UnsupportedFormat(String),

    #[error("Failed to open image: {0}")]
    ImageError(String),

    #[error("Failed to save thumbnail: {0}")]
    SaveError(String),

    #[error("Platform error: {0}")]
    PlatformError(String),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}
