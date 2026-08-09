//! Create a rolling tmux autosave archive suitable for an external scheduler.

use std::{
    path::{Path, PathBuf},
    str::FromStr,
};

use async_fs as fs;
use smol;
use smol::process::Command;
use tempfile::{NamedTempFile, TempDir};

use crate::{
    Result, actions::save::save_panes_content, error::Error, management::archive::v1, tmux,
};

/// Client information used to create a headless autosave and report it to Tmux.
#[derive(Debug, Clone)]
pub struct AutosaveContext {
    client: tmux::client::Client,
    tmux_target: Option<String>,
}

/// Determine the client context used by an autosave.
///
/// In Tmux, this uses the invoking client. Outside Tmux, such as from a scheduler, it selects the
/// most recently active attached client.
pub async fn context(require_tmux_target: bool) -> Result<AutosaveContext> {
    if std::env::var_os("TMUX").is_some() {
        let client = tmux::client::current().await?;
        let tmux_target = if require_tmux_target {
            Some(current_client_target().await?)
        } else {
            current_client_target().await.ok()
        };
        Ok(AutosaveContext {
            client,
            tmux_target,
        })
    } else {
        let tmux_target = most_recent_client_target().await?;
        let client = client_for_target(&tmux_target).await?;
        Ok(AutosaveContext {
            client,
            tmux_target: Some(tmux_target),
        })
    }
}

/// Display `message` in the selected Tmux client, if one is available.
///
/// Reporting is best effort: the primary error is still printed to stderr.
pub fn display_message(context: &AutosaveContext, message: &str) {
    let Some(target) = context.tmux_target.as_deref() else {
        return;
    };

    let _ = std::process::Command::new("tmux")
        .args(["display-message", "-t", target, message])
        .output();
}

/// Save the tmux sessions, windows and panes into the rolling autosave archive.
///
/// The archive is first fully created in a temporary file in `backup_dirpath`, then atomically
/// replaced at `autosave.tar.zst`.
pub async fn autosave<P: AsRef<Path>>(
    backup_dirpath: P,
    num_lines_to_drop: usize,
    context: AutosaveContext,
) -> Result<(PathBuf, v1::Overview)> {
    let backup_dirpath = backup_dirpath.as_ref();
    let temp_dir = TempDir::new()?;

    let metadata_task: smol::Task<Result<(PathBuf, PathBuf, u16, u16)>> = {
        let temp_dirpath = temp_dir.path().to_path_buf();
        let client = context.client;

        smol::spawn(async move {
            let temp_version_filepath = temp_dirpath.join(v1::VERSION_FILENAME);
            fs::write(&temp_version_filepath, v1::FORMAT_VERSION).await?;

            let metadata = v1::Metadata::new_with_client(client).await?;
            let json = serde_json::to_string(&metadata)?;

            let temp_metadata_filepath = temp_dirpath.join(v1::METADATA_FILENAME);
            fs::write(temp_metadata_filepath.as_path(), json).await?;

            Ok((
                temp_version_filepath,
                temp_metadata_filepath,
                metadata.sessions.len() as u16,
                metadata.windows.len() as u16,
            ))
        })
    };

    let (temp_panes_content_dir, num_panes) = {
        let temp_panes_content_dir = temp_dir.path().join(v1::PANES_DIR_NAME);
        fs::create_dir_all(&temp_panes_content_dir).await?;

        let panes = tmux::pane::available_panes().await?;
        let num_panes = panes.len() as u16;
        save_panes_content(panes, &temp_panes_content_dir, num_lines_to_drop).await?;

        (temp_panes_content_dir, num_panes)
    };
    let (temp_version_filepath, temp_metadata_filepath, num_sessions, num_windows) =
        metadata_task.await?;

    let autosave_filepath = v1::autosave_filepath(backup_dirpath);
    let temp_archive = NamedTempFile::new_in(backup_dirpath)?;
    let archive = temp_archive.reopen()?;
    v1::create_from_file(
        archive,
        &temp_version_filepath,
        &temp_metadata_filepath,
        &temp_panes_content_dir,
    )?;
    persist_autosave(temp_archive, &autosave_filepath)?;

    temp_dir.close()?;

    let overview = v1::Overview {
        version: v1::FORMAT_VERSION.to_string(),
        num_sessions,
        num_windows,
        num_panes,
    };

    Ok((autosave_filepath, overview))
}

/// Atomically replace the autosave archive with a completed temporary archive.
fn persist_autosave(temp_archive: NamedTempFile, autosave_filepath: &Path) -> Result<()> {
    temp_archive
        .persist(autosave_filepath)
        .map(|_| ())
        .map_err(|error| error.error.into())
}

async fn current_client_target() -> Result<String> {
    let output = Command::new("tmux")
        .args(["display-message", "-p", "-F", "#{client_name}"])
        .output()
        .await?;
    tmux_output(output, "could not determine the current Tmux client")
}

async fn most_recent_client_target() -> Result<String> {
    let output = Command::new("tmux")
        .args(["list-clients", "-F", "#{client_activity}\t#{client_name}"])
        .output()
        .await?;
    let output = tmux_output(output, "could not list Tmux clients")?;

    select_most_recent_client(&output)
        .ok_or_else(|| Error::ConfigError("no attached Tmux client available for autosave".into()))
}

fn select_most_recent_client(output: &str) -> Option<String> {
    output
        .lines()
        .filter_map(|line| {
            let (activity, target) = line.split_once('\t')?;
            Some((activity.parse::<u64>().ok()?, target))
        })
        .max_by_key(|(activity, _)| *activity)
        .map(|(_, target)| target.to_string())
}

async fn client_for_target(target: &str) -> Result<tmux::client::Client> {
    let output = Command::new("tmux")
        .args([
            "display-message",
            "-t",
            target,
            "-p",
            "-F",
            "'#{client_session}':'#{client_last_session}'",
        ])
        .output()
        .await?;
    let output = tmux_output(output, "could not read the selected Tmux client")?;

    tmux::client::Client::from_str(&output).map_err(|error| {
        Error::ConfigError(format!("could not parse selected Tmux client: {error}"))
    })
}

fn tmux_output(output: std::process::Output, failure_message: &str) -> Result<String> {
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        let message = if stderr.is_empty() {
            failure_message.to_string()
        } else {
            format!("{failure_message}: {stderr}")
        };
        return Err(Error::ConfigError(message));
    }

    String::from_utf8(output.stdout)
        .map(|output| output.trim_end().to_string())
        .map_err(|error| Error::ConfigError(format!("Tmux output was not valid UTF-8: {error}")))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn selects_most_recent_client() {
        let clients = "10\t/dev/ttys001\n22\t/dev/ttys002\n15\t/dev/ttys003\n";
        assert_eq!(
            select_most_recent_client(clients),
            Some("/dev/ttys002".into())
        );
    }

    #[test]
    fn ignores_malformed_client_rows() {
        let clients = "invalid\nnot-a-number\t/dev/ttys001\n25\t/dev/ttys002\n";
        assert_eq!(
            select_most_recent_client(clients),
            Some("/dev/ttys002".into())
        );
    }

    #[test]
    fn no_client_is_none() {
        assert_eq!(select_most_recent_client(""), None);
    }

    #[test]
    fn completed_temp_archive_replaces_previous_autosave() {
        use std::io::Write;

        let dir = TempDir::new().unwrap();
        let autosave_filepath = v1::autosave_filepath(dir.path());
        std::fs::write(&autosave_filepath, "previous archive").unwrap();

        let mut temp_archive = NamedTempFile::new_in(dir.path()).unwrap();
        temp_archive.write_all(b"new archive").unwrap();
        persist_autosave(temp_archive, &autosave_filepath).unwrap();

        assert_eq!(
            std::fs::read_to_string(autosave_filepath).unwrap(),
            "new archive"
        );
    }
}
