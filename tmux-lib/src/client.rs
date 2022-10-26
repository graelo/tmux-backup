//! Client-level functions: for representing client state (`client_session` etc) or reporting information inside Tmux.

use std::str::FromStr;

use async_std::process::Command;
use nom::{character::complete::char, combinator::all_consuming, sequence::tuple};
use serde::{Deserialize, Serialize};

use crate::{
    error::{map_add_intent, Error},
    parse::{quoted_nonempty_string, quoted_string},
    Result,
};

/// A Tmux client.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Client {
    /// The current session.
    pub session_name: String,
    /// The last session.
    pub last_session_name: String,
}

impl FromStr for Client {
    type Err = Error;

    /// Parse a string containing client information into a new `Client`.
    ///
    /// This returns a `Result<Client, Error>` as this call can obviously
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
    /// tmux display-message -p -F "'#{client_session}':'#{client_last_session}'"
    /// ```
    ///
    /// For definitions, look at `Pane` type and the tmux man page for
    /// definitions.
    fn from_str(input: &str) -> std::result::Result<Self, Self::Err> {
        let desc = "Client";
        let intent = "'#{client_session}':'#{client_last_session}'";
        let parser = tuple((quoted_nonempty_string, char(':'), quoted_string));

        let (_, (session_name, _, last_session_name)) =
            all_consuming(parser)(input).map_err(|e| map_add_intent(desc, intent, e))?;

        Ok(Client {
            session_name: session_name.to_string(),
            last_session_name: last_session_name.to_string(),
        })
    }
}

// ------------------------------
// Ops
// ------------------------------

/// Return the current client useful attributes.
///
/// # Errors
///
/// Returns an `io::IOError` in the command failed.
pub async fn current() -> Result<Client> {
    let args = vec![
        "display-message",
        "-p",
        "-F",
        "'#{client_session}':'#{client_last_session}'",
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

pub async fn switch_client(session_name: &str) -> Result<()> {
    let exact_session_name = format!("={session_name}");
    let args = vec!["switch-client", "-t", &exact_session_name];

    Command::new("tmux")
        .args(&args)
        .output()
        .await
        .expect("Cannot communicate with Tmux for switching the client");

    Ok(())
}
