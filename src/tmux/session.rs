//! This module provides a few types and functions to handle Tmux sessions.
//!
//! The main use cases are running Tmux commands & parsing Tmux session
//! information.

use std::str::FromStr;

use super::session_id::SessionId;
use crate::error::ParseError;

#[derive(Debug, PartialEq)]
pub struct Session {
    /// Session identifier, e.g. `$3`.
    pub id: SessionId,
    /// Name of the session.
    pub name: String,
}

impl FromStr for Session {
    type Err = ParseError;

    /// Parse a string containing tmux session status into a new `Session`.
    ///
    /// This returns a `Result<Session, ParseError>` as this call can obviously
    /// fail if provided an invalid format.
    ///
    /// The expected format of the tmux status is
    ///
    /// ```text
    /// $1:pytorch
    /// $2:rust
    /// $3:swift
    /// $4:tmux-hacking
    /// ```
    ///
    /// This status line is obtained with
    ///
    /// ```text
    /// tmux list-sessions -F "#{session_id}:#{session_name}"
    /// ```
    ///
    /// For definitions, look at `Session` type and the tmux man page for
    /// definitions.
    fn from_str(src: &str) -> Result<Self, Self::Err> {
        let items: Vec<&str> = src.split(':').collect();
        assert_eq!(items.len(), 2, "tmux should have returned 2 items per line");

        let mut iter = items.iter();

        // SessionId must be start with '%' followed by a `u32`
        let id_str = iter.next().unwrap();
        let id = SessionId::from_str(id_str)?;

        let name = iter.next().unwrap().to_string();

        Ok(Session { id, name })
    }
}

/// Returns a list of all `Session` from the current tmux session.
pub async fn available_sessions() -> Result<Vec<Session>, ParseError> {
    let args = vec!["list-sessions", "-F", "#{session_id}:#{session_name}"];

    let output = tokio::process::Command::new("tmux")
        .args(&args)
        .output()
        .await?;
    let buffer = String::from_utf8(output.stdout)?;

    // Each call to `Session::parse` returns a `Result<Session, _>`. All results
    // are collected into a Result<Vec<Session>, _>, thanks to `collect()`.
    let result: Result<Vec<Session>, ParseError> = buffer
        .trim_end() // trim last '\n' as it would create an empty line
        .split('\n')
        .map(|line| Session::from_str(line))
        .collect();

    result
}

#[cfg(test)]
mod tests {
    use super::Session;
    use super::SessionId;
    use crate::error;
    use std::str::FromStr;

    #[test]
    fn parse_list_sessions() {
        let output = vec!["$1:pytorch", "$2:rust", "$3:swift", "$4:tmux-hacking"];
        let sessions: Result<Vec<Session>, error::ParseError> =
            output.iter().map(|&line| Session::from_str(line)).collect();
        let sessions = sessions.expect("Could not parse tmux sessions");

        let expected = vec![
            Session {
                id: SessionId::from_str("$1").unwrap(),
                name: String::from("pytorch"),
            },
            Session {
                id: SessionId::from_str("$2").unwrap(),
                name: String::from("rust"),
            },
            Session {
                id: SessionId::from_str("$3").unwrap(),
                name: String::from("swift"),
            },
            Session {
                id: SessionId::from_str("$4").unwrap(),
                name: String::from("tmux-hacking"),
            },
        ];

        assert_eq!(sessions, expected);
    }
}
