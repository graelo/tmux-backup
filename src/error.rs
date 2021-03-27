use std::error::Error;
use std::fmt;

#[derive(Debug)]
pub enum ParseError {
    ExpectedIdMarker(char),
    ExpectedInt(std::num::ParseIntError),
    ExpectedBool(std::str::ParseBoolError),
    ProcessFailure(String),
}

impl Error for ParseError {
    fn description(&self) -> &str {
        "ParseError"
    }
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ParseError::ExpectedIdMarker(ch) => write!(f, "Expected id marker `{}`", ch),
            ParseError::ExpectedInt(msg) => write!(f, "Expected an int: {}", msg),
            ParseError::ExpectedBool(msg) => write!(f, "Expected a bool: {}", msg),
            ParseError::ProcessFailure(msg) => write!(f, "{}", msg),
        }
    }
}

impl From<std::num::ParseIntError> for ParseError {
    fn from(error: std::num::ParseIntError) -> Self {
        ParseError::ExpectedInt(error)
    }
}

impl From<std::str::ParseBoolError> for ParseError {
    fn from(error: std::str::ParseBoolError) -> Self {
        ParseError::ExpectedBool(error)
    }
}

impl From<std::io::Error> for ParseError {
    fn from(error: std::io::Error) -> Self {
        ParseError::ProcessFailure(error.to_string())
    }
}
