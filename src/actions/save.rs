//! Retrieve session information and panes content save to a backup.

use std::env;
use std::path::{Path, PathBuf};

use anyhow::Result;
use async_std::{fs, task};
use futures::future::join_all;

use crate::{
    management::{archive::v1, catalog::BackupDetails},
    tmux,
};

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
pub async fn save<P: AsRef<Path>>(backup_dirpath: P) -> Result<(PathBuf, BackupDetails)> {
    // Prepare the temp directory.
    let temp_dirpath = env::temp_dir().join("tmux-revive");
    fs::create_dir_all(&temp_dirpath).await?;

    // Save sessions & windows into `metadata.yaml` in the temp folder.
    let metadata_task: task::JoinHandle<Result<(PathBuf, PathBuf, u16, u16)>> = {
        let temp_dirpath = temp_dirpath.clone();

        task::spawn(async move {
            let temp_version_filepath = temp_dirpath.join(v1::VERSION_FILENAME);
            fs::write(&temp_version_filepath, v1::FORMAT_VERSION).await?;

            let sessions = tmux::session::available_sessions().await?;
            let windows = tmux::window::available_windows().await?;
            let panes = tmux::pane::available_panes().await?;
            let num_sessions = sessions.len() as u16;
            let num_windows = windows.len() as u16;

            let metadata = v1::Metadata {
                sessions,
                windows,
                panes,
            };
            let yaml = serde_yaml::to_string(&metadata)?;

            let temp_metadata_filepath = temp_dirpath.join(v1::METADATA_FILENAME);
            fs::write(temp_metadata_filepath.as_path(), yaml).await?;

            Ok((
                temp_version_filepath,
                temp_metadata_filepath,
                num_sessions,
                num_windows,
            ))
        })
    };

    // Save pane contents in the temp folder.
    let (temp_panes_content_dir, num_panes) = {
        let temp_panes_content_dir = temp_dirpath.join(v1::PANES_DIR_NAME);
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
    fs::remove_dir_all(temp_dirpath).await?;

    let details = BackupDetails {
        version: v1::FORMAT_VERSION.to_string(),
        num_sessions,
        num_windows,
        num_panes,
    };

    Ok((new_backup_filepath, details))
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
            let output = pane.capture().await.unwrap();

            let filename = format!("pane-{}.txt", pane.id);
            let filepath = dest_dir.join(filename);
            fs::write(filepath, output).await
        });
        handles.push(handle);
    }

    join_all(handles).await;
    Ok(())
}
