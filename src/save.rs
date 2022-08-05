//! Retrieve session information and panes content save to a backup.

use std::env;
use std::path::{Path, PathBuf};

use anyhow::Result;
use async_std::{fs, task};
use chrono::Local;
use futures::future::join_all;

use crate::tmux;
use crate::{Report, Summary, PANES_DIR_NAME, SUMMARY_FILENAME};

/// Save the tmux sessions, windows and panes into a backup at `backup_dirpath`.
///
/// The provided directory will be created if necessary. Backups have a name similar to
/// `backup-20220731T222948.tar.zst`.
///
/// The n-most recent backups are kept.
pub async fn save(backup_dirpath: &Path, rotate_size: usize) -> Result<Report> {
    fs::create_dir_all(&backup_dirpath).await?;

    let new_backup_filepath = {
        let timestamp_frag = Local::now().format("%Y%m%dT%H%M%S").to_string();
        let backup_filename = format!("backup-{timestamp_frag}.tar.zst");
        backup_dirpath.join(backup_filename)
    };

    // Prepare the temp directory.
    let temp_dirpath = env::temp_dir().join("tmux-revive");
    fs::create_dir_all(&temp_dirpath).await?;

    let summary_task: task::JoinHandle<Result<(PathBuf, u16, u16)>> = {
        let temp_dirpath = temp_dirpath.clone();

        task::spawn(async move {
            let sessions = tmux::session::available_sessions().await?;
            let windows = tmux::window::available_windows().await?;
            let num_sessions = sessions.len() as u16;
            let num_windows = windows.len() as u16;

            let summary = Summary { sessions, windows };
            let yaml = serde_yaml::to_string(&summary)?;

            let temp_summary_filepath = temp_dirpath.join(SUMMARY_FILENAME);
            fs::write(temp_summary_filepath.as_path(), yaml).await?;

            Ok((temp_summary_filepath, num_sessions, num_windows))
        })
    };

    let (temp_panes_content_dir, num_panes) = {
        let temp_panes_content_dir = temp_dirpath.join(PANES_DIR_NAME);
        fs::create_dir_all(&temp_panes_content_dir).await?;

        let panes = tmux::pane::available_panes().await?;
        let num_panes = panes.len() as u16;
        save_panes_content(panes, &temp_panes_content_dir).await?;

        (temp_panes_content_dir, num_panes)
    };
    let (temp_summary_filepath, num_sessions, num_windows) = summary_task.await?;

    create_archive(
        &new_backup_filepath,
        &temp_summary_filepath,
        &temp_panes_content_dir,
    )?;

    // fs::remove_dir_all(temp_panes_content_dir).await?;
    fs::remove_dir_all(temp_dirpath).await?;

    let report = Report {
        num_sessions,
        num_windows,
        num_panes,
    };

    Ok(report)
}

/// For each provided pane, retrieve the content and save it into `destination_dir`.
async fn save_panes_content(panes: Vec<tmux::pane::Pane>, destination_dir: &Path) -> Result<()> {
    let mut handles = Vec::new();

    for pane in panes {
        let dest_dir = destination_dir.to_path_buf();
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

fn create_archive(
    archive_filepath: &Path,
    summary_filepath: &Path,
    panes_content_dir: &Path,
) -> Result<()> {
    // println!("compressing content of {:?}", panes_content_dir);
    let archive = std::fs::File::create(archive_filepath)?;
    let enc = zstd::stream::write::Encoder::new(archive, 0)?.auto_finish();
    let mut tar = tar::Builder::new(enc);

    // println!("appending {:?}", summary_filepath);
    tar.append_path_with_name(summary_filepath, SUMMARY_FILENAME)?;
    // println!("appending {:?}", panes_content_dir);
    tar.append_dir_all(PANES_DIR_NAME, panes_content_dir)?;
    tar.finish()?;

    Ok(())
}
