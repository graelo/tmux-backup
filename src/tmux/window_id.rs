use std::fmt;
use std::str::FromStr;

use serde::{Deserialize, Serialize};

use crate::error;

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct WindowId(String);

impl FromStr for WindowId {
    type Err = error::ParseError;

    /// Parse into WindowId. The `&str` must start with '@' followed by a
    /// `u16`.
    fn from_str(src: &str) -> Result<Self, Self::Err> {
        if !src.starts_with('@') {
            return Err(error::ParseError::ExpectedIdMarker('@'));
        }
        let id = src[1..].parse::<u16>()?;
        let id = format!("@{}", id);
        Ok(WindowId(id))
    }
}

// impl WindowId {
//     pub fn as_str(&self) -> &str {
//         &self.0
//     }
// }

impl fmt::Display for WindowId {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}
