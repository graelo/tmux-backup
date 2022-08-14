//! Server management.

use async_std::process::Command;

use crate::error::ParseError;

/// Start the Tmux server if needed, creating the `"[placeholder]"` session to keep the server
/// running.
///
/// It is ok-ish to already have an existing session named `"[placeholder]"`.
pub async fn start() -> Result<(), ParseError> {
    let args = vec!["new-session", "-d", "-s", "[placeholder]"];

    let output = Command::new("tmux").args(&args).output().await?;
    let buffer = String::from_utf8(output.stdout)?;

    if buffer.is_empty() || buffer.contains("duplicate") {
        return Ok(());
    }
    Err(ParseError::UnexpectedOutput(buffer))
}
