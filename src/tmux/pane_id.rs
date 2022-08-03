use std::fmt;
use std::str::FromStr;

use crate::error;
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct PaneId(pub String);

impl FromStr for PaneId {
    type Err = error::ParseError;

    /// Parse into PaneId. The `&str` must start with '%' followed by a `u32`.
    fn from_str(src: &str) -> Result<Self, Self::Err> {
        if !src.starts_with('%') {
            return Err(error::ParseError::ExpectedIdMarker('$'));
        }
        let id = src[1..].parse::<u16>()?;
        let id = format!("%{}", id);
        Ok(PaneId(id))
    }
}

impl PaneId {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for PaneId {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}
