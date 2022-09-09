//! Window Id.

use std::str::FromStr;

use nom::{
    character::complete::{char, digit1},
    combinator::all_consuming,
    sequence::preceded,
    IResult,
};
use serde::{Deserialize, Serialize};

use crate::error::{map_add_intent, Error};

/// The id of a Tmux window.
///
/// This wraps the raw tmux representation (`@41`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WindowId(String);

impl FromStr for WindowId {
    type Err = Error;

    /// Parse into WindowId. The `&str` must start with '@' followed by a
    /// `u16`.
    fn from_str(input: &str) -> std::result::Result<Self, Self::Err> {
        let desc = "WindowId";
        let intent = "#{window_id}";

        let (_, window_id) =
            all_consuming(parse::window_id)(input).map_err(|e| map_add_intent(desc, intent, e))?;

        Ok(window_id)
    }
}

impl WindowId {
    /// Extract a string slice containing the raw representation.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

pub(crate) mod parse {
    use super::*;

    pub(crate) fn window_id(input: &str) -> IResult<&str, WindowId> {
        let (input, digit) = preceded(char('@'), digit1)(input)?;
        let id = format!("@{}", digit);
        Ok((input, WindowId(id)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_window_id_fn() {
        let actual = parse::window_id("@43");
        let expected = Ok(("", WindowId("@43".into())));
        assert_eq!(actual, expected);

        let actual = parse::window_id("@4");
        let expected = Ok(("", WindowId("@4".into())));
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_parse_window_id_struct() {
        let actual = WindowId::from_str("@43");
        assert!(actual.is_ok());
        assert_eq!(actual.unwrap(), WindowId("@43".into()));

        let actual = WindowId::from_str("4:38");
        assert!(matches!(
            actual,
            Err(Error::ParseError {
                desc: "WindowId",
                intent: "#{window_id}",
                err: _
            })
        ));
    }
}
