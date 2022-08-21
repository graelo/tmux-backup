//! Server management.

use std::collections::HashMap;

use async_std::process::Command;

use crate::{error::Error, Result};

/// Start the Tmux server if needed, creating a session named `"[placeholder]"` in order to keep the server
/// running.
///
/// It is ok-ish to already have an existing session named `"[placeholder]"`.
pub async fn start(initial_session_name: &str) -> Result<()> {
    let args = vec!["new-session", "-d", "-s", initial_session_name];

    let output = Command::new("tmux").args(&args).output().await?;
    let buffer = String::from_utf8(output.stdout)?;

    if buffer.is_empty() || buffer.contains("duplicate") {
        return Ok(());
    }
    Err(Error::UnexpectedOutput(buffer))
}

/// Remove the session named `"[placeholder]"` used to keep the server alive.
pub async fn kill_session(name: &str) -> Result<()> {
    let exact_name = format!("={name}");
    let args = vec!["kill-session", "-t", &exact_name];

    let output = Command::new("tmux").args(&args).output().await?;
    let buffer = String::from_utf8(output.stdout)?;

    if buffer.is_empty() {
        return Ok(());
    }
    Err(Error::UnexpectedOutput(buffer))
}

/// Return the value of a Tmux option. For instance, this can be used to get Tmux's default
/// command.
pub async fn show_option(option_name: &str, global: bool) -> Result<Option<String>> {
    let mut args = vec!["show-options", "-w", "-q"];
    if global {
        args.push("-g");
    }
    args.push(option_name);

    let output = Command::new("tmux").args(&args).output().await?;
    let buffer = String::from_utf8(output.stdout)?;
    let buffer = buffer.trim_end();

    if buffer.is_empty() {
        return Ok(None);
    }
    Ok(Some(buffer.to_string()))
}

/// Return all Tmux options as a `std::haosh::HashMap`.
pub async fn show_options(global: bool) -> Result<HashMap<String, String>> {
    let args = if global {
        vec!["show-options", "-g"]
    } else {
        vec!["show-options"]
    };

    let output = Command::new("tmux").args(&args).output().await?;
    let buffer = String::from_utf8(output.stdout)?;
    let pairs: HashMap<String, String> = buffer
        .trim_end()
        .split('\n')
        .into_iter()
        .map(|s| s.split_at(s.find(' ').unwrap()))
        .map(|(k, v)| (k.to_string(), v[1..].to_string()))
        .collect();

    Ok(pairs)
}

/// Return the `"default-command"` used to start a pane, falling back to `"default shell"` if none.
///
/// In case of bash, a `-l` flag is added.
pub async fn default_command() -> Result<String> {
    let all_options = show_options(true).await?;

    let default_shell = all_options
        .get("default-shell")
        .ok_or(Error::TmuxConfig("no default-shell"))
        .map(|cmd| cmd.to_owned())
        .map(|cmd| {
            if cmd.ends_with("bash") {
                format!("-l {}", cmd)
            } else {
                cmd
            }
        })?;

    all_options
        .get("default-command")
        .or(Some(&default_shell))
        .ok_or(Error::TmuxConfig("no default-command nor default-shell"))
        .map(|cmd| cmd.to_owned())
}
