//! Functions for reporting information inside Tmux.

use std::process::Command;

/// Returns a list of all `Pane` from all sessions.
///
/// # Panics
///
/// This function panics if it cannot communicate with Tmux.
pub fn display_message(message: &str) {
    let args = vec!["display-message", message];

    Command::new("tmux")
        .args(&args)
        .output()
        .expect("Cannot communicate with Tmux for displaying message");
}
