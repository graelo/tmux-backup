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

pub struct Archive {
    version: String,
    metadata: Metadata,
}

impl Archive {
    /// Open the archive file then read the version string and tmux metadata.
    pub async fn read_file<P: AsRef<Path>>(backup_filepath: P) -> Result<Archive> {
        let archive = std::fs::File::open(backup_filepath.as_ref())?;
        let dec = zstd::stream::read::Decoder::new(archive)?;
        let mut tar = tar::Archive::new(dec);

        // Read the version file.
        let mut version = String::new();
        version.reserve(4);

        let mut bytes = vec![];
        bytes.reserve(8 * 1024);

        for mut entry in tar.entries()?.flatten() {
            if entry.path().unwrap().to_string_lossy() == VERSION_FILENAME {
                entry.read_to_string(&mut version)?;
            } else if entry.path().unwrap().to_string_lossy() == METADATA_FILENAME {
                entry.read_to_end(&mut bytes)?;
            }
        }

        if version.is_empty() {
            return Err(anyhow::anyhow!("Could not read the format version"));
        }
        if bytes.is_empty() {
            return Err(anyhow::anyhow!("Could not read metadata"));
        }

        let metadata = serde_yaml::from_slice(&bytes)?;

        Ok(Archive { version, metadata })
    }

    pub fn overview(&self) -> BackupOverview {
        BackupOverview {
            version: self.version.clone(),
            num_sessions: self.metadata.sessions.len() as u16,
            num_windows: self.metadata.windows.len() as u16,
            num_panes: self.metadata.panes.len() as u16,
        }
    }

    pub fn full_description(&self) -> String {
        "full description of the archive with session names".into()
    }
}

/// Return the filepath for a new backup.
///
/// This is used when the function ``actions::save`` needs a new filepath.
pub fn new_backup_filepath<P>(dirpath: P) -> PathBuf
where
    P: AsRef<Path>,
{
    let timestamp_frag = Local::now().format("%Y%m%dT%H%M%S").to_string();
    let backup_filename = format!("backup-{timestamp_frag}.tar.zst");
    dirpath.as_ref().join(backup_filename)
}

/// Create a new backup file in `dest_filepath` with the contents of the metadata file and panes
/// content.
pub fn create_from_paths<P: AsRef<Path>>(
    dest_filepath: P,
    version_filepath: P,
    metadata_filepath: P,
    panes_content_dir: P,
) -> Result<()> {
    let archive = std::fs::File::create(dest_filepath.as_ref())?;
    let enc = zstd::stream::write::Encoder::new(archive, 0)?.auto_finish();
    let mut tar = tar::Builder::new(enc);

    tar.append_path_with_name(version_filepath, VERSION_FILENAME)?;
    tar.append_path_with_name(metadata_filepath.as_ref(), METADATA_FILENAME)?;
    tar.append_dir_all(PANES_DIR_NAME, panes_content_dir.as_ref())?;
    tar.finish()?;

    Ok(())
}
