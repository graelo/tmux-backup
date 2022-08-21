//! This module provides a few types and functions to handle Tmux sessions.
//!
//! The main use cases are running Tmux commands & parsing Tmux session
//! information.

use async_std::process::Command;
use std::{path::PathBuf, str::FromStr};

use serde::{Deserialize, Serialize};

use super::{
    pane::Pane, pane_id::PaneId, session_id::SessionId, window::Window, window_id::WindowId,
};
use crate::error::ParseError;

/// A Tmux session.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Session {
    /// Session identifier, e.g. `$3`.
    pub id: SessionId,
    /// Name of the session.
    pub name: String,
    /// Working directory of the session.
    pub dirpath: PathBuf,
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
    /// $1:pytorch:/Users/graelo/dl/pytorch
    /// $2:rust:/Users/graelo/rust
    /// $3:swift:/Users/graelo/swift
    /// $4:tmux-hacking:/Users/graelo/tmux
    /// ```
    ///
    /// This status line is obtained with
    ///
    /// ```text
    /// tmux list-sessions -F "#{session_id}:#{session_name}:#{session_path}"
    /// ```
    ///
    /// For definitions, look at `Session` type and the tmux man page for
    /// definitions.
    fn from_str(src: &str) -> Result<Self, Self::Err> {
        let items: Vec<&str> = src.split(':').collect();
        if items.len() != 3 {
            return Err(ParseError::UnexpectedOutput(src.into()));
        }
        let mut iter = items.iter();

        // SessionId must be start with '%' followed by a `u16`
        let id_str = iter.next().unwrap();
        let id = SessionId::from_str(id_str)?;

        let name = iter.next().unwrap().to_string();

        let dirpath = iter.next().unwrap().into();

        Ok(Session { id, name, dirpath })
    }
}

/// Return a list of all `Session` from the current tmux session.
pub async fn available_sessions() -> Result<Vec<Session>, ParseError> {
    let args = vec![
        "list-sessions",
        "-F",
        "#{session_id}:#{session_name}:#{session_path}",
    ];

    let output = Command::new("tmux").args(&args).output().await?;
    let buffer = String::from_utf8(output.stdout)?;

    // Each call to `Session::parse` returns a `Result<Session, _>`. All results
    // are collected into a Result<Vec<Session>, _>, thanks to `collect()`.
    let result: Result<Vec<Session>, ParseError> = buffer
        .trim_end() // trim last '\n' as it would create an empty line
        .split('\n')
        .map(Session::from_str)
        .collect();

    result
}

/// Create a Tmux session (and thus a window & pane).
///
/// The new session attributes:
///
/// - the session name is taken from the passed `session`
/// - the working directory is taken from the pane's working directory.
///
pub async fn new_session(
    session: &Session,
    window: &Window,
    pane: &Pane,
    pane_command: Option<&str>,
) -> Result<(SessionId, WindowId, PaneId), ParseError> {
    let mut args = vec![
        "new-session",
        "-d",
        "-c",
        pane.dirpath.to_str().unwrap(),
        "-s",
        &session.name,
        "-n",
        &window.name,
        "-P",
        "-F",
        "#{session_id}:#{window_id}:#{pane_id}",
    ];
    if let Some(pane_command) = pane_command {
        args.push(pane_command);
    }

    let output = Command::new("tmux").args(&args).output().await?;
    let buffer = String::from_utf8(output.stdout)?;

    let items: Vec<&str> = buffer.trim_end().split(':').collect();
    assert_eq!(items.len(), 3);

    let mut iter = items.iter();

    let id_str = iter.next().unwrap();
    let new_session_id = SessionId::from_str(id_str)?;

    let id_str = iter.next().unwrap();
    let new_window_id = WindowId::from_str(id_str)?;

    let id_str = iter.next().unwrap();
    let new_pane_id = PaneId::from_str(id_str)?;

    Ok((new_session_id, new_window_id, new_pane_id))
}

#[cfg(test)]
mod tests {
    use super::Session;
    use super::SessionId;
    use crate::error;
    use std::path::PathBuf;
    use std::str::FromStr;

    #[test]
    fn parse_list_sessions() {
        let output = vec![
            "$1:pytorch:/Users/graelo/ml/pytorch",
            "$2:rust:/Users/graelo/rust",
            "$3:swift:/Users/graelo/swift",
            "$4:tmux-hacking:/Users/graelo/tmux",
        ];
        let sessions: Result<Vec<Session>, error::ParseError> =
            output.iter().map(|&line| Session::from_str(line)).collect();
        let sessions = sessions.expect("Could not parse tmux sessions");

        let expected = vec![
            Session {
                id: SessionId::from_str("$1").unwrap(),
                name: String::from("pytorch"),
                dirpath: PathBuf::from("/Users/graelo/ml/pytorch"),
            },
            Session {
                id: SessionId::from_str("$2").unwrap(),
                name: String::from("rust"),
                dirpath: PathBuf::from("/Users/graelo/rust"),
            },
            Session {
                id: SessionId::from_str("$3").unwrap(),
                name: String::from("swift"),
                dirpath: PathBuf::from("/Users/graelo/swift"),
            },
            Session {
                id: SessionId::from_str("$4").unwrap(),
                name: String::from("tmux-hacking"),
                dirpath: PathBuf::from("/Users/graelo/tmux"),
            },
        ];

        assert_eq!(sessions, expected);
    }
}
