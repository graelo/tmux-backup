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

// impl fmt::Display for ParseError {
//     fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
//         match self {
//             ParseError::ExpectedIdMarker(ch) => write!(f, "Expected id marker `{}`", ch),
//             ParseError::ExpectedInt(msg) => write!(f, "Expected an int: {}", msg),
//             ParseError::ExpectedBool(msg) => write!(f, "Expected a bool: {}", msg),
//             ParseError::ProcessFailure(msg) => write!(f, "{}", msg),
//         }
//     }
// }

// impl From<std::num::ParseIntError> for ParseError {
//     fn from(error: std::num::ParseIntError) -> Self {
//         ParseError::ExpectedInt(error)
//     }
// }

// impl From<std::str::ParseBoolError> for ParseError {
//     fn from(error: std::str::ParseBoolError) -> Self {
//         ParseError::ExpectedBool(error)
//     }
// }

// impl From<std::io::Error> for ParseError {
//     fn from(error: std::io::Error) -> Self {
//         ParseError::ProcessFailure(error.to_string())
//     }
// }

// impl From<FromUtf8Error> for ParseError {
//     fn from(error: std::string::FromUtf8Error) -> Self {
//         ParseError::ProcessFailure(error.to_string())
//     }
// }
