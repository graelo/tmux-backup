//! Server management.

use async_std::process::Command;

use crate::error::ParseError;

/// Start the Tmux server if needed, creating a session named `"[placeholder]"` in order to keep the server
/// running.
///
/// It is ok-ish to already have an existing session named `"[placeholder]"`.
pub async fn start(initial_session_name: &str) -> Result<(), ParseError> {
    let args = vec!["new-session", "-d", "-s", initial_session_name];

    let output = Command::new("tmux").args(&args).output().await?;
    let buffer = String::from_utf8(output.stdout)?;

    if buffer.is_empty() || buffer.contains("duplicate") {
        return Ok(());
    }
    Err(ParseError::UnexpectedOutput(buffer))
}

/// Remove the session named `"[placeholder]"` used to keep the server alive.
pub async fn kill_session(name: &str) -> Result<(), ParseError> {
    let exact_name = format!("={name}");
    let args = vec!["kill-session", "-t", &exact_name];

    let output = Command::new("tmux").args(&args).output().await?;
    let buffer = String::from_utf8(output.stdout)?;

    if buffer.is_empty() {
        return Ok(());
    }
    Err(ParseError::UnexpectedOutput(buffer))
}
