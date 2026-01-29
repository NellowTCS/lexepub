use async_zip::error::ZipError;
use thiserror::Error;

/// Main error type for lexepub operations
#[derive(Debug, Error)]
pub enum LexEpubError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("ZIP error: {0}")]
    Zip(#[from] ZipError),

    #[error("XML parsing error: {0}")]
    Xml(#[from] quick_xml::Error),

    #[error("HTML parsing error: {0}")]
    Html(String),

    #[error("Invalid EPUB format: {0}")]
    InvalidFormat(String),

    #[error("Missing required file: {0}")]
    MissingFile(String),

    #[error("Metadata parsing error: {0}")]
    MetadataError(String),

    #[error("Chapter parsing error: {0}")]
    ChapterError(String),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("UTF-8 error: {0}")]
    Utf8(#[from] std::string::FromUtf8Error),

    #[error("UTF-8 conversion error: {0}")]
    Utf8Str(#[from] std::str::Utf8Error),

    #[error("Async task error: {0}")]
    AsyncError(String),
}

/// Result type for convenience
pub type Result<T> = std::result::Result<T, LexEpubError>;
