//! Support functions to create and read backup archive files.

use std::fmt;
use std::path::{Path, PathBuf};

use anyhow::Result;
use chrono::Local;
use serde::{Deserialize, Serialize};

use crate::tmux;

/// Name of the directory storing the panes content in the backup.
///
/// This name is also used in the temporary directory when retrieving the panes content from Tmux.
pub const PANES_DIR_NAME: &str = "panes-content";

/// Name of the file storing the metadata in the backup.
///
/// This name is also used in the temporary directory when storing the catalog.
pub const METADATA_FILENAME: &str = "metadata.yaml";

/// Describes the number of sessions, windows and panes in a backup.
///
/// This report is displayed after the commands `save`, `restore`, or `catalog describe`.
#[derive(Debug)]
pub struct Report {
    /// Number of sessions in a backup.
    pub num_sessions: u16,

    /// Number of windows in a backup.
    pub num_windows: u16,

    /// Number of panes in a backup.
    pub num_panes: u16,
}

impl fmt::Display for Report {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!(
            "{} sessions ({} windows, {} panes)",
            self.num_sessions, self.num_windows, self.num_panes,
        ))
    }
}

/// Describes the Tmux sessions, windows & panes metadata to store in a backup.
///
/// This is enough information to recreate all sessions, windows & panes.
#[derive(Debug, Serialize, Deserialize)]
pub struct Metadata {
    /// Tmux sessions metadata.
    pub sessions: Vec<tmux::session::Session>,

    /// Tmux windows metadata.
    pub windows: Vec<tmux::window::Window>,
}

/// Return the filepath for a new backup.
///
/// This is used when the `actions::save::save` method needs a new filepath.
pub fn new_backup_filepath<P>(dirpath: P) -> PathBuf
where
    P: AsRef<Path>,
{
    let timestamp_frag = Local::now().format("%Y%m%dT%H%M%S").to_string();
    let backup_filename = format!("backup-{timestamp_frag}.tar.zst");
    dirpath.as_ref().join(backup_filename)
}

/// Create a new backup file in `backup_filepath` with the contents of the metadata file and panes
/// content.
pub fn create<P: AsRef<Path>>(
    backup_filepath: P,
    metadata_filepath: P,
    panes_content_dir: P,
) -> Result<()> {
    let archive = std::fs::File::create(backup_filepath.as_ref())?;
    let enc = zstd::stream::write::Encoder::new(archive, 0)?.auto_finish();
    let mut tar = tar::Builder::new(enc);

    // println!("appending {:?}", metadata_filepath);
    tar.append_path_with_name(metadata_filepath.as_ref(), METADATA_FILENAME)?;
    // println!("appending {:?}", panes_content_dir);
    tar.append_dir_all(PANES_DIR_NAME, panes_content_dir.as_ref())?;
    tar.finish()?;

    Ok(())
}
