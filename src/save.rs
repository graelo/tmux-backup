//! Retrieve session information and panes content save to an archive.

use std::env;
use std::path::{Path, PathBuf};

use anyhow::Result;
use async_std::{fs, task};
use chrono::Local;
use futures::future::join_all;

use crate::tmux;
use crate::{Catalog, Report, CATALOG_FILENAME, PANES_DIR_NAME};

/// Save the tmux sessions, windows and panes into an archive at `archive_dirpath`.
///
/// The provided directory will be created if necessary. Archives have a name similar to
/// `archive-20220731T222948.tar.zst`.
///
/// The n-most recent archives are kept.
pub async fn save(archive_dirpath: &Path, num_archives: u16) -> Result<Report> {
    fs::create_dir_all(&archive_dirpath).await?;

    let archive_filepath = {
        let timestamp_frag = Local::now().format("%Y%m%dT%H%M%S").to_string();
        let archive_filename = format!("archive-{timestamp_frag}.tar.zst");
        archive_dirpath.join(archive_filename)
    };

    // Prepare the temp directory.
    let temp_dirpath = env::temp_dir().join("tmux-revive");
    fs::create_dir_all(&temp_dirpath).await?;

    let catalog_task: task::JoinHandle<Result<(PathBuf, u16, u16)>> = {
        let temp_dirpath = temp_dirpath.clone();

        task::spawn(async move {
            let sessions = tmux::session::available_sessions().await?;
            let windows = tmux::window::available_windows().await?;
            let num_sessions = sessions.len() as u16;
            let num_windows = windows.len() as u16;

            let catalog = Catalog { sessions, windows };
            let yaml = serde_yaml::to_string(&catalog)?;

            let temp_catalog_filepath = temp_dirpath.join(CATALOG_FILENAME);
            fs::write(temp_catalog_filepath.as_path(), yaml).await?;

            Ok((temp_catalog_filepath, num_sessions, num_windows))
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
    let (temp_catalog_filepath, num_sessions, num_windows) = catalog_task.await?;

    create_archive(
        &archive_filepath,
        &temp_catalog_filepath,
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
    catalog_filepath: &Path,
    panes_content_dir: &Path,
) -> Result<()> {
    // println!("compressing content of {:?}", panes_content_dir);
    let archive = std::fs::File::create(archive_filepath)?;
    let enc = zstd::stream::write::Encoder::new(archive, 0)?.auto_finish();
    let mut tar = tar::Builder::new(enc);

    // println!("appending {:?}", catalog_filepath);
    tar.append_path_with_name(catalog_filepath, CATALOG_FILENAME)?;
    // println!("appending {:?}", panes_content_dir);
    tar.append_dir_all(PANES_DIR_NAME, panes_content_dir)?;
    tar.finish()?;

    Ok(())
}
