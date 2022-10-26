//! Session Id.

use std::str::FromStr;

use nom::{
    character::complete::{char, digit1},
    combinator::all_consuming,
    sequence::preceded,
    IResult,
};
use serde::{Deserialize, Serialize};

use crate::error::{map_add_intent, Error};

/// The id of a Tmux session.
///
/// This wraps the raw tmux representation (`$11`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SessionId(String);

impl FromStr for SessionId {
    type Err = Error;

    /// Parse into SessionId. The `&str` must start with '$' followed by a
    /// `u16`.
    fn from_str(input: &str) -> std::result::Result<Self, Self::Err> {
        let desc = "SessionId";
        let intent = "#{session_id}";

        let (_, sess_id) =
            all_consuming(parse::session_id)(input).map_err(|e| map_add_intent(desc, intent, e))?;

        Ok(sess_id)
    }
}

// impl SessionId {
//     pub fn as_str(&self) -> &str {
//         &self.0
//     }
// }

pub(crate) mod parse {
    use super::*;

    pub fn session_id(input: &str) -> IResult<&str, SessionId> {
        let (input, digit) = preceded(char('$'), digit1)(input)?;
        let id = format!("${digit}");
        Ok((input, SessionId(id)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_session_id_fn() {
        let actual = parse::session_id("$43");
        let expected = Ok(("", SessionId("$43".into())));
        assert_eq!(actual, expected);

        let actual = parse::session_id("$4");
        let expected = Ok(("", SessionId("$4".into())));
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_parse_session_id_struct() {
        let actual = SessionId::from_str("$43");
        assert!(actual.is_ok());
        assert_eq!(actual.unwrap(), SessionId("$43".into()));

        let actual = SessionId::from_str("4:38");
        assert!(matches!(
            actual,
            Err(Error::ParseError {
                desc: "SessionId",
                intent: "#{session_id}",
                err: _
            })
        ));
    }
}
