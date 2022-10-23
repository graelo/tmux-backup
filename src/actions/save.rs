//! Retrieve session information and panes content save to a backup.

use std::path::{Path, PathBuf};

use async_std::{fs, task};
use futures::future::join_all;
use tempfile::TempDir;

use crate::{management::archive::v1, tmux, Result};

/// Save the tmux sessions, windows and panes into a backup at `backup_dirpath`.
///
/// After saving, this function returns the path to the backup and the number of
/// sessions, windows and panes.
///
/// # Notes
///
/// - The `backup_dirpath` folder is assumed to exist (done during catalog initialization).
/// - Backups have a name similar to `backup-20220731T222948.tar.zst`.
///
pub async fn save<P: AsRef<Path>>(backup_dirpath: P) -> Result<(PathBuf, v1::Overview)> {
    // Prepare the temp directory.
    let temp_dir = TempDir::new()?;

    // Save sessions & windows into `metadata.json` in the temp folder.
    let metadata_task: task::JoinHandle<Result<(PathBuf, PathBuf, u16, u16)>> = {
        let temp_dirpath = temp_dir.path().to_path_buf();

        task::spawn(async move {
            let temp_version_filepath = temp_dirpath.join(v1::VERSION_FILENAME);
            fs::write(&temp_version_filepath, v1::FORMAT_VERSION).await?;

            let metadata = v1::Metadata::new().await?;

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

    // Save pane contents in the temp folder.
    let (temp_panes_content_dir, num_panes) = {
        let temp_panes_content_dir = temp_dir.path().join(v1::PANES_DIR_NAME);
        fs::create_dir_all(&temp_panes_content_dir).await?;

        let panes = tmux::pane::available_panes().await?;
        let num_panes = panes.len() as u16;
        save_panes_content(panes, &temp_panes_content_dir).await?;

        (temp_panes_content_dir, num_panes)
    };
    let (temp_version_filepath, temp_metadata_filepath, num_sessions, num_windows) =
        metadata_task.await?;

    // Tar-compress content of temp folder into a new backup file in `backup_dirpath`.
    let new_backup_filepath = v1::new_backup_filepath(backup_dirpath.as_ref());

    v1::create_from_paths(
        &new_backup_filepath,
        &temp_version_filepath,
        &temp_metadata_filepath,
        &temp_panes_content_dir,
    )?;

    // Cleanup the entire temp folder.
    temp_dir.close()?;

    let overview = v1::Overview {
        version: v1::FORMAT_VERSION.to_string(),
        num_sessions,
        num_windows,
        num_panes,
    };

    Ok((new_backup_filepath, overview))
}

/// For each provided pane, retrieve the content and save it into `destination_dir`.
async fn save_panes_content<P: AsRef<Path>>(
    panes: Vec<tmux::pane::Pane>,
    destination_dir: P,
) -> Result<()> {
    let mut handles = Vec::new();

    for pane in panes {
        let dest_dir = destination_dir.as_ref().to_path_buf();
        let handle = task::spawn(async move {
            let should_drop_last_line = pane.command == "zsh";
            let output = pane.capture(should_drop_last_line).await.unwrap();

            let filename = format!("pane-{}.txt", pane.id);
            let filepath = dest_dir.join(filename);
            fs::write(filepath, output).await
        });
        handles.push(handle);
    }

    join_all(handles).await;
    Ok(())
}
