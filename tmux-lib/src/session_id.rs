//! Session Id.

use std::fmt;
use std::str::FromStr;

use nom::{
    character::complete::{char, digit1},
    sequence::preceded,
    IResult,
};
use serde::{Deserialize, Serialize};

use crate::error::Error;

/// The id of a Tmux session.
///
/// This wraps the raw tmux representation (`$11`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SessionId(String);

impl FromStr for SessionId {
    type Err = Error;

    /// Parse into SessionId. The `&str` must start with '$' followed by a
    /// `u16`.
    fn from_str(src: &str) -> std::result::Result<Self, Self::Err> {
        // if let Ok((input, sess_id)) = session_id(src) && input.is_empty(){
        //     return Ok(sess_id);
        // }
        // Err(Error::ParseSessionIdError(src.into()))

        match session_id(src) {
            Ok((input, sess_id)) => {
                if input.is_empty() {
                    Ok(sess_id)
                } else {
                    Err(Error::ParseSessionIdError(src.into()))
                }
            }
            Err(_) => Err(Error::ParseSessionIdError(src.into())),
        }
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

pub fn session_id(input: &str) -> IResult<&str, SessionId> {
    let (input, digit) = preceded(char('$'), digit1)(input)?;
    let id = format!("${}", digit);
    Ok((input, SessionId(id)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_session_id_fn() {
        let actual = session_id("$43");
        let expected = Ok(("", SessionId("$43".into())));
        assert_eq!(actual, expected);

        let actual = session_id("$4");
        let expected = Ok(("", SessionId("$4".into())));
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_parse_session_id_struct() {
        let actual = SessionId::from_str("$43");
        assert!(actual.is_ok());
        assert_eq!(actual.unwrap(), SessionId("$43".into()));

        let actual = SessionId::from_str("4:38");
        assert!(matches!(actual, Err(Error::ParseSessionIdError(string)) if string == *"4:38"));
    }
}
