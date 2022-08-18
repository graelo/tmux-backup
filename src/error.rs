use std::io;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum ParseError {
    #[error("expected a tmux id marker `{0}`")]
    ExpectedIdMarker(char),

    #[error("failed parsing int")]
    ExpectedInt(#[from] std::num::ParseIntError),

    #[error("failed parsing bool")]
    ExpectedBool(#[from] std::str::ParseBoolError),

    #[error("unexpected process output: `{0}`")]
    UnexpectedOutput(String),

    #[error("unexpected tmux config: `{0}`")]
    TmuxConfig(&'static str),

    // #[error("process failed with error `{0}`")]
    // ProcessFailure(String),
    #[error("failed parsing utf-8 string: `{source}`")]
    Utf8 {
        #[from]
        source: std::string::FromUtf8Error,
    },

    #[error("failed with io: `{source}`")]
    Io {
        #[from]
        source: io::Error,
    },
}
