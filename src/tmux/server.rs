//! Server management.

use async_std::process::Command;

use crate::error::ParseError;

/// Name of the placeholder session.
const PLACEHOLDER_SESSION_NAME: &str = "[placeholder]";

/// Start the Tmux server if needed, creating a session named `"[placeholder]"` in order to keep the server
/// running.
///
/// It is ok-ish to already have an existing session named `"[placeholder]"`.
pub async fn start() -> Result<(), ParseError> {
    let args = vec!["new-session", "-d", "-s", PLACEHOLDER_SESSION_NAME];

    let output = Command::new("tmux").args(&args).output().await?;
    let buffer = String::from_utf8(output.stdout)?;

    if buffer.is_empty() || buffer.contains("duplicate") {
        return Ok(());
    }
    Err(ParseError::UnexpectedOutput(buffer))
}

/// Remove the session named `"[placeholder]"` used to keep the server alive.
pub async fn kill_placeholder_session() -> Result<(), ParseError> {
    let args = vec!["kill-session", "-t", PLACEHOLDER_SESSION_NAME];

    let output = Command::new("tmux").args(&args).output().await?;
    let buffer = String::from_utf8(output.stdout)?;

    if buffer.is_empty() {
        return Ok(());
    }
    Err(ParseError::UnexpectedOutput(buffer))
}
