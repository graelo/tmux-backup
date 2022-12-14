//! This crate's error type.

use std::io;

use crate::tmux;

/// Describes all errors from this crate.
///
/// - errors during backup operations.
/// - errors reported by tmux
#[derive(thiserror::Error, Debug)]
pub enum Error {
    /// Errors occuring during tmux operations.
    #[error("Tmux operation failed: `{source}`")]
    TmuxError {
        #[from]
        /// Source error.
        source: tmux::error::Error,
    },

    /// Unsupported archive version.
    #[error("unsupported archive version: `{0}`")]
    ArchiveVersion(String),

    /// Backup file contains no metadata.
    #[error("missing metadata: `{0}`")]
    MissingMetadata(String),

    /// Configuration error.
    #[error("unexpected configuration: `{0}`")]
    ConfigError(String),

    /// Serde error.
    #[error("serde error: `{source}`")]
    Serde {
        #[from]
        /// Source error,
        source: serde_json::Error,
    },

    /// Some IO error.
    #[error("failed with io: `{source}`")]
    Io {
        #[from]
        /// Source error.
        source: io::Error,
    },
}
