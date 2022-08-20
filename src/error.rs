//! This crate's error type.

use std::io;

/// Describes all errors variants from this crate.
#[derive(thiserror::Error, Debug)]
pub enum ParseError {
    /// Failed parsing a tmux id marker for sessions, windows or panes.
    #[error("expected a tmux id marker `{0}`")]
    ExpectedIdMarker(char),

    /// Failed parsing an integer from a tmux response.
    #[error("failed parsing int")]
    ExpectedInt(#[from] std::num::ParseIntError),

    /// Failed parsing a bool from a tmux response.
    #[error("failed parsing bool")]
    ExpectedBool(#[from] std::str::ParseBoolError),

    /// A tmux invocation returned some output where none was expected (actions such as
    /// some `tmux display-message` invocations).
    #[error("unexpected process output: `{0}`")]
    UnexpectedOutput(String),

    /// Indicates Tmux has a weird config, like missing the `"default-shell"`.
    #[error("unexpected tmux config: `{0}`")]
    TmuxConfig(&'static str),

    /// Failed parsing the output of a process invocation as utf-8.
    #[error("failed parsing utf-8 string: `{source}`")]
    Utf8 {
        #[from]
        /// Source error.
        source: std::string::FromUtf8Error,
    },

    /// Some IO error.
    #[error("failed with io: `{source}`")]
    Io {
        #[from]
        /// Source error.
        source: io::Error,
    },
}

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
        source: ParseError,
    },

    /// Unsupported archive version.
    #[error("unsupported archive version: `{0}`")]
    ArchiveVersion(String),

    /// Backup file contains no metadata.
    #[error("missing metadata: `{0}`")]
    MissingMetadata(String),

    /// Serde error.
    #[error("serde error: `{source}`")]
    Serde {
        #[from]
        /// Source error,
        source: serde_yaml::Error,
    },

    /// Some IO error.
    #[error("failed with io: `{source}`")]
    Io {
        #[from]
        /// Source error.
        source: io::Error,
    },
}
