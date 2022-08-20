//! Client-level functions: for representing client state (`client_session` etc) or reporting information inside Tmux.

use std::str::FromStr;

use async_std::process::Command;
use serde::{Deserialize, Serialize};

use crate::error;

/// A Tmux client.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Client {
    /// The current session.
    pub session_name: String,
    /// The last session.
    pub last_session_name: String,
}

impl FromStr for Client {
    type Err = error::ParseError;

    /// Parse a string containing client information into a new `Client`.
    ///
    /// This returns a `Result<Client, ParseError>` as this call can obviously
    /// fail if provided an invalid format.
    ///
    /// The expected format of the tmux response is
    ///
    /// ```text
    /// name-of-current-session:name-of-last-session
    /// ```
    ///
    /// This status line is obtained with
    ///
    /// ```text
    /// tmux display-message -p -F "#{client_session}:#{client_last_session}"
    /// ```
    ///
    /// For definitions, look at `Pane` type and the tmux man page for
    /// definitions.
    fn from_str(src: &str) -> Result<Self, Self::Err> {
        let items: Vec<&str> = src.split(':').collect();
        if items.len() != 2 {
            return Err(error::ParseError::UnexpectedOutput(src.into()));
        }

        let mut iter = items.iter();

        // Session id must be start with '$' followed by a `u16`
        let session_name = iter.next().unwrap().to_string();
        let last_session_name = iter.next().unwrap().to_string();

        Ok(Client {
            session_name,
            last_session_name,
        })
    }
}

/// Return the current client useful attributes.
pub async fn current_client() -> Result<Client, error::ParseError> {
    let args = vec![
        "display-message",
        "-p",
        "-F",
        "#{client_session}:#{client_last_session}",
    ];

    let output = Command::new("tmux").args(&args).output().await?;
    let buffer = String::from_utf8(output.stdout)?;

    Client::from_str(buffer.trim_end())
}

/// Return a list of all `Pane` from all sessions.
///
/// # Panics
///
/// This function panics if it can't communicate with Tmux.
pub fn display_message(message: &str) {
    let args = vec!["display-message", message];

    std::process::Command::new("tmux")
        .args(&args)
        .output()
        .expect("Cannot communicate with Tmux for displaying message");
}

/// Switch to session exactly named `session_name`.

pub async fn switch_client(session_name: &str) -> Result<(), error::ParseError> {
    let exact_session_name = format!("={session_name}");
    let args = vec!["switch-client", "-t", &exact_session_name];

    Command::new("tmux")
        .args(&args)
        .output()
        .await
        .expect("Cannot communicate with Tmux for switching the client");

    Ok(())
}
