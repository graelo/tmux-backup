//! Support functions to create and read backup archive files.

use std::io::Read;
use std::path::{Path, PathBuf};

use anyhow::Result;
use chrono::Local;
use serde::{Deserialize, Serialize};

use crate::management::catalog::BackupOverview;
use crate::tmux;

/// Version of the archive format.
pub const FORMAT_VERSION: &str = "1.0";

/// Name of the file storing the version of the archive format.
pub const VERSION_FILENAME: &str = "version";

/// Name of the directory storing the panes content in the backup.
///
/// This name is also used in the temporary directory when retrieving the panes content from Tmux.
pub const PANES_DIR_NAME: &str = "panes-content";

/// Name of the file storing the metadata in the backup.
///
/// This name is also used in the temporary directory when storing the catalog.
pub const METADATA_FILENAME: &str = "metadata.yaml";

/// Describes the Tmux sessions, windows & panes metadata to store in a backup.
///
/// This is enough information to recreate all sessions, windows & panes.
#[derive(Debug, Serialize, Deserialize)]
pub struct Metadata {
    /// Tmux sessions metadata.
    pub sessions: Vec<tmux::session::Session>,

    /// Tmux windows metadata.
    pub windows: Vec<tmux::window::Window>,

    /// Tmux panes metadata.
    pub panes: Vec<tmux::pane::Pane>,
}

impl Metadata {
    pub fn read<P: AsRef<Path>>(backup_filepath: P) -> Result<Metadata> {
        read_metadata(backup_filepath)
    }

    pub fn get_overview(&self) -> BackupOverview {
        // let panes = self.windows.iter().flat_map(|w| w.)
        BackupOverview {
            num_sessions: self.sessions.len() as u16,
            num_windows: self.windows.len() as u16,
            num_panes: self.panes.len() as u16,
        }
    }
}

/// Return the filepath for a new backup.
///
/// This is used when the method ``actions::save::save`` needs a new filepath.
pub fn new_backup_filepath<P>(dirpath: P) -> PathBuf
where
    P: AsRef<Path>,
{
    let timestamp_frag = Local::now().format("%Y%m%dT%H%M%S").to_string();
    let backup_filename = format!("backup-{timestamp_frag}.tar.zst");
    dirpath.as_ref().join(backup_filename)
}

/// Read the metadata from a backup file.
///
/// This function is used in `catalog list --details` and `catalog describe`.
pub fn read_metadata<P: AsRef<Path>>(backup_filepath: P) -> Result<Metadata> {
    let archive = std::fs::File::open(backup_filepath.as_ref())?;
    let dec = zstd::stream::read::Decoder::new(archive)?;
    let mut tar = tar::Archive::new(dec);

    let mut bytes = vec![];
    bytes.reserve(8 * 1024);

    let n_bytes = tar
        .entries()?
        .filter_map(|e| e.ok())
        .find(|entry| entry.path().unwrap().to_string_lossy() == METADATA_FILENAME)
        .map(|mut entry| entry.read_to_end(&mut bytes));

    if n_bytes.is_none() {
        return Err(anyhow::anyhow!("Could not read metadata"));
    }

    let metadata = serde_yaml::from_slice(&bytes)?;
    Ok(metadata)
}

/// Create a new backup file in `backup_filepath` with the contents of the metadata file and panes
/// content.
pub fn create<P: AsRef<Path>>(
    backup_filepath: P,
    version_filepath: P,
    metadata_filepath: P,
    panes_content_dir: P,
) -> Result<()> {
    let archive = std::fs::File::create(backup_filepath.as_ref())?;
    let enc = zstd::stream::write::Encoder::new(archive, 0)?.auto_finish();
    let mut tar = tar::Builder::new(enc);

    tar.append_path_with_name(version_filepath, "VERSION_FILENAME")?;
    tar.append_path_with_name(metadata_filepath.as_ref(), METADATA_FILENAME)?;
    tar.append_dir_all(PANES_DIR_NAME, panes_content_dir.as_ref())?;
    tar.finish()?;

    Ok(())
}
