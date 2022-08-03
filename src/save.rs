//! Retrieve session information and panes content save to an archive.

use std::env;
use std::path::PathBuf;

use async_std::{fs, task};
use chrono::Local;
use futures::future::join_all;

use crate::tmux;
use crate::{Catalog, Result, CATALOG_FILENAME, PANES_DIR_NAME};

/// Save the tmux sessions, windows and panes into an archive at `archive_dirpath`.
///
/// The provided directory will be created if necessary. Archives have a name similar to
/// `archive-20220731T222948.tar.zst`.
pub async fn save(archive_dirpath: &PathBuf) -> Result<()> {
    fs::create_dir_all(&archive_dirpath).await?;

    let archive_filepath = {
        let timestamp_frag = Local::now().format("%Y%m%dT%H%M%S").to_string();
        let archive_filename = format!("archive-{timestamp_frag}.tar.zst");
        archive_dirpath.join(archive_filename)
    };

    // Prepare the temp directory.
    let temp_dirpath = env::temp_dir().join("tmux-revive");
    fs::create_dir_all(&temp_dirpath).await?;

    let catalog_task: task::JoinHandle<Result<PathBuf>> = {
        let temp_dirpath = temp_dirpath.clone();

        task::spawn(async move {
            let sessions = tmux::session::available_sessions().await?;
            let windows = tmux::window::available_windows().await?;
            let catalog = Catalog { sessions, windows };

            let yaml = serde_yaml::to_string(&catalog)?;

            let temp_catalog_filepath = temp_dirpath.join(CATALOG_FILENAME);
            fs::write(temp_catalog_filepath.as_path(), yaml).await?;

            Ok(temp_catalog_filepath)
        })
    };

    let temp_panes_content_dir = {
        let temp_panes_content_dir = temp_dirpath.join(PANES_DIR_NAME);
        fs::create_dir_all(&temp_panes_content_dir).await?;

        let panes = tmux::pane::available_panes().await?;
        save_panes_content(panes, &temp_panes_content_dir).await?;

        temp_panes_content_dir
    };
    let temp_catalog_filepath = catalog_task.await?;

    create_archive(
        &archive_filepath,
        &temp_catalog_filepath,
        &temp_panes_content_dir,
    )?;

    // fs::remove_dir_all(temp_panes_content_dir).await?;
    fs::remove_dir_all(temp_dirpath).await?;

    Ok(())
}

/// For each provided pane, retrieve the content and save it into `destination_dir`.
async fn save_panes_content(panes: Vec<tmux::pane::Pane>, destination_dir: &PathBuf) -> Result<()> {
    let mut handles = Vec::new();

    for pane in panes {
        let dest_dir = destination_dir.clone();
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
    archive_filepath: &PathBuf,
    catalog_filepath: &PathBuf,
    panes_content_dir: &PathBuf,
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
