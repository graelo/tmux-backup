use std::io;

/// Describes all errors variants from this crate.
#[derive(thiserror::Error, Debug)]
pub enum Error {
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

    /// Failed parsing a PaneId.
    #[error("failed parsing PaneId from `{0}`")]
    ParsePaneIdError(String),

    /// Failed parsing a Pane.
    #[error("failed parsing Pane from `{0}`")]
    ParsePaneError(String),

    /// Failed parsing a WindowId.
    #[error("failed parsing WindowId from `{0}`")]
    ParseWindowIdError(String),

    /// Failed parsing a Window.
    #[error("failed parsing Window from `{0}`")]
    ParseWindowError(String),

    /// Failed parsing a SessionId.
    #[error("failed parsing SessionId from `{0}`")]
    ParseSessionIdError(String),

    /// Failed parsing a Session.
    #[error("failed parsing Session from `{0}`")]
    ParseSessionError(String),

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
