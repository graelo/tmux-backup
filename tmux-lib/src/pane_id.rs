//! Pane id.

use std::fmt;
use std::str::FromStr;

use nom::{
    character::complete::{char, digit1},
    combinator::all_consuming,
    sequence::preceded,
    IResult,
};
use serde::{Deserialize, Serialize};

use crate::error::{map_add_intent, Error};

/// The id of a Tmux pane.
///
/// This wraps the raw tmux representation (`%12`).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PaneId(pub String);

impl FromStr for PaneId {
    type Err = Error;

    /// Parse into PaneId. The `&str` must start with '%' followed by a `u32`.
    fn from_str(input: &str) -> Result<Self, Self::Err> {
        let desc = "PaneId";
        let intent = "#{pane_id}";

        let (_, pane_id) =
            all_consuming(parse::pane_id)(input).map_err(|e| map_add_intent(desc, intent, e))?;

        Ok(pane_id)
    }
}

impl From<&u16> for PaneId {
    fn from(value: &u16) -> Self {
        Self(format!("%{value}"))
    }
}

impl PaneId {
    /// Extract a string slice containing the raw representation.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for PaneId {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

pub(crate) mod parse {
    use super::*;

    pub(crate) fn pane_id(input: &str) -> IResult<&str, PaneId> {
        let (input, digit) = preceded(char('%'), digit1)(input)?;
        let id = format!("%{digit}");
        Ok((input, PaneId(id)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_pane_id_fn() {
        let actual = parse::pane_id("%43");
        let expected = Ok(("", PaneId("%43".into())));
        assert_eq!(actual, expected);

        let actual = parse::pane_id("%4");
        let expected = Ok(("", PaneId("%4".into())));
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_parse_pane_id_struct() {
        let actual = PaneId::from_str("%43");
        assert!(actual.is_ok());
        assert_eq!(actual.unwrap(), PaneId("%43".into()));

        let actual = PaneId::from_str("4:38");
        assert!(matches!(
            actual,
            Err(Error::ParseError {
                desc: "PaneId",
                intent: "#{pane_id}",
                err: _
            })
        ));
    }
}
