//! Support functions to create and read backup archive files.

use std::collections::HashSet;
use std::fmt;
use std::io::Read;
use std::path::{Path, PathBuf};

use chrono::Local;
use serde::{Deserialize, Serialize};

use crate::{error::Error, tmux, Result};

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
pub const METADATA_FILENAME: &str = "metadata.json";

/// Describes the Tmux sessions, windows & panes stored in a backup.
///
/// This is enough information to recreate all sessions, windows & panes.
#[derive(Debug, Serialize, Deserialize)]
pub struct Metadata {
    /// Version of the archive's format.
    pub version: String,

    /// Tmux client metadata.
    pub client: tmux::client::Client,

    /// Tmux sessions metadata.
    pub sessions: Vec<tmux::session::Session>,

    /// Tmux windows metadata.
    pub windows: Vec<tmux::window::Window>,

    /// Tmux panes metadata.
    pub panes: Vec<tmux::pane::Pane>,
}

impl Metadata {
    /// Query Tmux and return a new `Metadata`.
    pub async fn new() -> Result<Self> {
        let version = FORMAT_VERSION.to_string();
        let client = tmux::client::current().await?;
        let sessions = tmux::session::available_sessions().await?;
        let windows = tmux::window::available_windows().await?;
        let panes = tmux::pane::available_panes().await?;

        let metadata = Self {
            version,
            client,
            sessions,
            windows,
            panes,
        };

        Ok(metadata)
    }

    /// Open the archive file at `backup_filepath` and read the version string and tmux metadata.
    pub async fn read_file<P: AsRef<Path>>(backup_filepath: P) -> Result<Self> {
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
                if version.is_empty() {
                    return Err(Error::ArchiveVersion(
                        "could not read the format version".to_string(),
                    ));
                }
                if version != FORMAT_VERSION {
                    return Err(Error::ArchiveVersion(format!(
                        "Unsupported format version: `{}`",
                        version
                    )));
                }
            } else if entry.path().unwrap().to_string_lossy() == METADATA_FILENAME {
                entry.read_to_end(&mut bytes)?;
            }
        }

        if bytes.is_empty() {
            return Err(Error::MissingMetadata(format!(
                "missing metadata in `{}`",
                backup_filepath.as_ref().to_string_lossy()
            )));
        }

        let metadata = serde_json::from_slice(&bytes)?;

        Ok(metadata)
    }

    /// Return an overview of the metadata.
    pub fn overview(&self) -> Overview {
        Overview {
            version: self.version.clone(),
            num_sessions: self.sessions.len() as u16,
            num_windows: self.windows.len() as u16,
            num_panes: self.panes.len() as u16,
        }
    }

    /// Return the list of windows in the provided session.
    pub fn windows_related_to(
        &self,
        session: &tmux::session::Session,
    ) -> Vec<tmux::window::Window> {
        self.windows
            .iter()
            .filter(|&w| w.sessions.contains(&session.name))
            .cloned()
            .collect()
    }

    /// Return the list of panes in the provided window.
    pub fn panes_related_to(&self, window: &tmux::window::Window) -> Vec<&tmux::pane::Pane> {
        let pane_ids: HashSet<tmux::pane_id::PaneId> = window.pane_ids().iter().cloned().collect();
        self.panes
            .iter()
            .filter(|&p| pane_ids.contains(&p.id))
            .collect()
    }
}

/// Overview of the archive's content: number of sessions, windows and panes in the archive.
///
/// These counts are displayed after the commands such as `save`, `restore`, or `catalog list
/// --details`.
#[derive(Debug)]
pub struct Overview {
    /// Format version of the archive.
    pub version: String,

    /// Number of sessions in the archive.
    pub num_sessions: u16,

    /// Number of windows in the archive.
    pub num_windows: u16,

    /// Number of panes in the archive.
    pub num_panes: u16,
}

impl fmt::Display for Overview {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!(
            "{} sessions {} windows {} panes",
            self.num_sessions, self.num_windows, self.num_panes,
        ))
    }
}

/// Print a full description of the archive, with session and window names.
pub async fn print_description<P>(_backup_filepath: P) -> Result<()>
where
    P: AsRef<Path>,
{
    unimplemented!()
    // let metadata = read_metadata(backup_filepath).await?;
    // let overview = metadata.overview();

    // println!("full details {overview}");

    // Ok(())
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

/// Unpack a backup at `backup_filepath` into `dest_dirpath`.
///
/// This is used to unpack the archive into `/tmp/` and access the panes-content.
pub async fn unpack<P: AsRef<Path>>(
    backup_filepath: P,
    dest_dirpath: P,
) -> std::result::Result<(), std::io::Error> {
    let archive = std::fs::File::open(backup_filepath.as_ref())?;
    let dec = zstd::stream::read::Decoder::new(archive)?;
    let mut tar = tar::Archive::new(dec);

    tar.unpack(dest_dirpath)
}
