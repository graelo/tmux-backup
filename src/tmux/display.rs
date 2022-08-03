use std::process::Command;

use crate::error;

/// Returns a list of all `Pane` from all sessions.
pub fn display_message(message: &str) -> Result<(), error::ParseError> {
    let args = vec!["display-message", message];

    Command::new("tmux").args(&args).output()?;
    // let buffer = String::from_utf8(output.stdout)?;
    Ok(())
}
