//! Errors returned by core operations.

use std::path::PathBuf;

/// Error returned when parsing a UI reference identifier.
#[derive(Debug, thiserror::Error)]
#[error("invalid UI reference identifier `{value}`; expected ui_<ULID>")]
pub struct ReferenceIdError {
    pub(crate) value: String,
}

impl ReferenceIdError {
    pub(crate) fn new(value: impl Into<String>) -> Self {
        Self {
            value: value.into(),
        }
    }
}

/// Error returned when converting DOM coordinates into screenshot pixels.
#[derive(Debug, thiserror::Error)]
pub enum CropError {
    /// The transform contained a zero or non-finite dimension.
    #[error("invalid coordinate transform: {0}")]
    InvalidTransform(&'static str),
    /// The selected element does not intersect the captured image.
    #[error("the selected element is outside the captured window")]
    OutsideCapture,
}

/// Error returned by reference persistence operations.
#[derive(Debug, thiserror::Error)]
pub enum StorageError {
    /// The requested reference does not exist.
    #[error("UI reference `{id}` was not found")]
    NotFound {
        /// Normalized identifier that was requested.
        id: String,
    },
    /// A filesystem operation failed.
    #[error("filesystem operation failed for `{path}`: {source}")]
    Io {
        /// Path involved in the failed operation.
        path: PathBuf,
        /// Underlying filesystem error.
        #[source]
        source: std::io::Error,
    },
    /// Reference JSON could not be encoded or decoded.
    #[error("reference JSON operation failed: {0}")]
    Json(#[from] serde_json::Error),
    /// A PNG could not be written.
    #[error("reference screenshot could not be encoded: {0}")]
    Image(#[from] image::ImageError),
}

impl StorageError {
    pub(crate) fn io(path: impl Into<PathBuf>, source: std::io::Error) -> Self {
        Self::Io {
            path: path.into(),
            source,
        }
    }
}
