//! Create a rolling tmux autosave archive suitable for an external scheduler.

use std::fs;
use std::path::{Path, PathBuf};
use std::thread;

use tempfile::{NamedTempFile, TempDir};

use crate::{
    Result, actions::save::save_panes, error::Error, management::archive::v1, tmux, tmux::Tmux,
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
pub fn context(tmux: &Tmux, require_tmux_target: bool) -> Result<AutosaveContext> {
    if std::env::var_os("TMUX").is_some() {
        let client = tmux.current_client()?.ok_or_else(|| {
            Error::ConfigError("running inside TMUX but tmux reported no client".into())
        })?;
        let tmux_target = if require_tmux_target {
            Some(tmux.current_client_name()?)
        } else {
            tmux.current_client_name().ok()
        };
        Ok(AutosaveContext {
            client,
            tmux_target,
        })
    } else {
        let tmux_target = tmux.most_recent_client_name()?.ok_or_else(|| {
            Error::ConfigError("no attached Tmux client available for autosave".into())
        })?;
        let client = tmux.client_for_target(&tmux_target)?;
        Ok(AutosaveContext {
            client,
            tmux_target: Some(tmux_target),
        })
    }
}

/// Display `message` in the selected Tmux client, if one is available.
///
/// Reporting is best effort: the primary error is still printed to stderr.
pub fn display_message(tmux: &Tmux, context: &AutosaveContext, message: &str) {
    let Some(target) = context.tmux_target.as_deref() else {
        return;
    };

    let _ = tmux.display_message_to(target, message);
}

/// Save the tmux sessions, windows and panes into the rolling autosave archive.
///
/// The archive is first fully created in a temporary file in `backup_dirpath`, then atomically
/// replaced at `autosave.tar.zst`.
pub fn autosave<P: AsRef<Path>>(
    tmux: &Tmux,
    backup_dirpath: P,
    num_lines_to_drop: usize,
    context: AutosaveContext,
) -> Result<(PathBuf, v1::Overview)> {
    let backup_dirpath = backup_dirpath.as_ref();
    let temp_dir = TempDir::new()?;
    let temp_panes_content_dir = temp_dir.path().join(v1::PANES_DIR_NAME);
    fs::create_dir_all(&temp_panes_content_dir)?;

    // Query sessions & windows on their own thread, so that the pane captures
    // overlap with it.
    let (metadata, panes) = thread::scope(|scope| {
        let client = context.client;
        let metadata = scope.spawn(move || v1::Metadata::new_with_client(tmux, client));
        let panes = save_panes(tmux, &temp_panes_content_dir, num_lines_to_drop);
        (metadata.join().expect("metadata query panicked"), panes)
    });
    let num_panes = panes?;
    let metadata = metadata?;

    let temp_version_filepath = temp_dir.path().join(v1::VERSION_FILENAME);
    fs::write(&temp_version_filepath, v1::FORMAT_VERSION)?;

    let temp_metadata_filepath = temp_dir.path().join(v1::METADATA_FILENAME);
    fs::write(&temp_metadata_filepath, serde_json::to_string(&metadata)?)?;

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
        num_sessions: metadata.sessions.len() as u16,
        num_windows: metadata.windows.len() as u16,
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

#[cfg(test)]
mod tests {
    use super::*;

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
