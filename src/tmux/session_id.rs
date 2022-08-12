use std::fmt;
use std::str::FromStr;

use serde::{Deserialize, Serialize};

use crate::error::ParseError;

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SessionId(String);

impl FromStr for SessionId {
    type Err = ParseError;

    /// Parse into SessionId. The `&str` must start with '$' followed by a
    /// `u16`.
    fn from_str(src: &str) -> Result<Self, Self::Err> {
        if !src.starts_with('$') {
            return Err(ParseError::ExpectedIdMarker('$'));
        }
        let id = src[1..].parse::<u16>()?;
        let id = format!("${}", id);
        Ok(SessionId(id))
    }
}

// impl SessionId {
//     pub fn as_str(&self) -> &str {
//         &self.0
//     }
// }

impl fmt::Display for SessionId {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}
