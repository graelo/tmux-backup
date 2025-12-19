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

        let mut bytes = Vec::with_capacity(8 * 1024);

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
                        "Unsupported format version: `{version}`",
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

/// Return the pattern for searching the backup files.
///
/// This is called by the catalog command to list the available backups.
///
/// # Note
///
/// This pattern must match the filename generated by `new_backup_filepath()`.
pub fn backup_filepath_pattern() -> &'static str {
    r".*backup-(\d{8}T\d{6})\.\d{6}\.tar\.zst"
}

/// Return the filepath for a new backup.
///
/// This is used when the function `actions::save` needs a new filepath. The filepath is based on
/// the current timestamp and is read by the catalog using the function `backup_filepath_pattern()`.
pub fn new_backup_filepath<P>(dirpath: P) -> PathBuf
where
    P: AsRef<Path>,
{
    let timestamp_frag = Local::now().format("%Y%m%dT%H%M%S%.6f").to_string();
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

#[cfg(test)]
mod tests {
    use super::*;
    use regex::Regex;

    mod backup_filepath_pattern {
        use super::*;

        fn matches(path: &str) -> bool {
            let pattern = backup_filepath_pattern();
            Regex::new(pattern).unwrap().is_match(path)
        }

        fn extract_timestamp(path: &str) -> Option<String> {
            let pattern = backup_filepath_pattern();
            let re = Regex::new(pattern).unwrap();
            re.captures(path).map(|c| c[1].to_string())
        }

        #[test]
        fn matches_standard_backup_filename() {
            assert!(matches("backup-20220910T172024.141993.tar.zst"));
        }

        #[test]
        fn matches_with_absolute_path() {
            assert!(matches(
                "/home/user/.local/state/tmux-backup/backup-20220910T172024.141993.tar.zst"
            ));
        }

        #[test]
        fn matches_with_relative_path() {
            assert!(matches("./backups/backup-20220910T172024.141993.tar.zst"));
        }

        #[test]
        fn extracts_timestamp_without_microseconds() {
            let ts = extract_timestamp("backup-20220910T172024.141993.tar.zst");
            assert_eq!(ts, Some("20220910T172024".to_string()));
        }

        #[test]
        fn rejects_missing_extension() {
            assert!(!matches("backup-20220910T172024.141993.tar"));
            assert!(!matches("backup-20220910T172024.141993"));
        }

        #[test]
        fn rejects_wrong_prefix() {
            assert!(!matches("snapshot-20220910T172024.141993.tar.zst"));
            assert!(!matches("20220910T172024.141993.tar.zst"));
        }

        #[test]
        fn rejects_malformed_timestamp() {
            // Missing T separator
            assert!(!matches("backup-20220910172024.141993.tar.zst"));
            // Wrong date format
            assert!(!matches("backup-2022-09-10T17:20:24.141993.tar.zst"));
            // Too short
            assert!(!matches("backup-20220910T1720.141993.tar.zst"));
        }

        #[test]
        fn rejects_missing_microseconds() {
            assert!(!matches("backup-20220910T172024.tar.zst"));
        }

        #[test]
        fn accepts_various_valid_timestamps() {
            // Midnight
            assert!(matches("backup-20240101T000000.000000.tar.zst"));
            // End of day
            assert!(matches("backup-20241231T235959.999999.tar.zst"));
            // Leap year date
            assert!(matches("backup-20240229T120000.123456.tar.zst"));
        }
    }

    mod new_backup_filepath {
        use super::*;

        #[test]
        fn generates_path_in_given_directory() {
            let path = new_backup_filepath("/my/backup/dir");
            assert!(path.starts_with("/my/backup/dir"));
        }

        #[test]
        fn generated_filename_has_correct_extension() {
            let path = new_backup_filepath("/tmp");
            let filename = path.file_name().unwrap().to_string_lossy();
            assert!(filename.ends_with(".tar.zst"));
        }

        #[test]
        fn generated_filename_starts_with_backup() {
            let path = new_backup_filepath("/tmp");
            let filename = path.file_name().unwrap().to_string_lossy();
            assert!(filename.starts_with("backup-"));
        }

        #[test]
        fn generated_path_matches_pattern() {
            let path = new_backup_filepath("/tmp");
            let pattern = backup_filepath_pattern();
            let re = Regex::new(pattern).unwrap();
            assert!(re.is_match(&path.to_string_lossy()));
        }

        #[test]
        fn accepts_path_with_trailing_slash() {
            let path = new_backup_filepath("/tmp/");
            assert!(path.starts_with("/tmp"));
        }

        #[test]
        fn works_with_pathbuf() {
            let dir = PathBuf::from("/var/backups");
            let path = new_backup_filepath(dir);
            assert!(path.starts_with("/var/backups"));
        }
    }

    mod overview_display {
        use super::*;

        #[test]
        fn formats_counts_correctly() {
            let overview = Overview {
                version: "1.0".to_string(),
                num_sessions: 3,
                num_windows: 12,
                num_panes: 47,
            };

            let output = format!("{overview}");
            assert_eq!(output, "3 sessions 12 windows 47 panes");
        }

        #[test]
        fn handles_singular_counts() {
            let overview = Overview {
                version: "1.0".to_string(),
                num_sessions: 1,
                num_windows: 1,
                num_panes: 1,
            };

            // Note: The current implementation doesn't pluralize
            let output = format!("{overview}");
            assert_eq!(output, "1 sessions 1 windows 1 panes");
        }

        #[test]
        fn handles_zero_counts() {
            let overview = Overview {
                version: "1.0".to_string(),
                num_sessions: 0,
                num_windows: 0,
                num_panes: 0,
            };

            let output = format!("{overview}");
            assert_eq!(output, "0 sessions 0 windows 0 panes");
        }
    }

    mod constants {
        use super::*;

        #[test]
        fn format_version_is_semver_like() {
            // Ensure version looks like "X.Y" or similar
            assert!(FORMAT_VERSION.contains('.'));
            assert!(!FORMAT_VERSION.is_empty());
        }

        #[test]
        fn panes_dir_name_is_reasonable() {
            assert!(!PANES_DIR_NAME.is_empty());
            assert!(!PANES_DIR_NAME.contains('/'));
            assert!(!PANES_DIR_NAME.contains('\\'));
        }

        #[test]
        fn metadata_filename_is_json() {
            assert!(METADATA_FILENAME.ends_with(".json"));
        }
    }
}
