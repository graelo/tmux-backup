mod error;
mod tmux;

use async_std::{fs, task};
use chrono::Local;
use futures::future::join_all;
use serde::{Deserialize, Serialize};
use std::env;
use std::path::PathBuf;
use tmux::session::Session;
use tmux::window::Window;

/// Name of the in-tar directory storing the panes content.
///
/// This name is also used in the temporary directory when retrieving the panes content from Tmux.
const PANES_DIR_NAME: &str = "panes-content";

/// Name of the in-tar file storing the catalog.
const CATALOG_FILENAME: &str = "catalog.yaml";

// Just a generic Result type to ease error handling for us. Errors in multithreaded
// async contexts needs some extra restrictions
type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

struct Opts {
    /// Directory where to save the archive.
    ///
    /// The archive name will be `archive-20220531T123456.tar.zst`, located under that path.
    archive_dirpath: PathBuf,
}

#[derive(Debug, Serialize, Deserialize)]
struct Catalog {
    sessions: Vec<Session>,
    windows: Vec<Window>,
}

async fn save() -> Result<()> {
    // Config
    //
    let archive_dirpath: PathBuf = {
        let state_home = match env::var("XDG_STATE_HOME") {
            Ok(v) => PathBuf::from(v),
            Err(_) => match env::var("HOME") {
                Ok(v) => PathBuf::from(v).join(".local").join("state"),
                Err(_) => PathBuf::from("/tmp").join("state"),
            },
        };
        state_home.join("tmux-revive")
    };
    let opts = Opts { archive_dirpath };

    fs::create_dir_all(&opts.archive_dirpath).await?;

    let archive_filepath = {
        // 20220731T222948
        let timestamp_frag = Local::now().format("%Y%m%dT%H%M%S").to_string();
        let archive_filename = format!("archive-{timestamp_frag}.tar.zst");
        opts.archive_dirpath.join(archive_filename)
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

fn main() {
    match task::block_on(save()) {
        Ok(_) => println!("✅ sessions persisted."),
        Err(e) => println!("An error ocurred: {}", e),
    };
}
