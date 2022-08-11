//! High-level backup metadata useful for catalog operations and reporting.

use std::fmt;
use std::path::PathBuf;

use chrono::NaiveDateTime;
use clap::ValueEnum;

/// Quick access, high-level backup metadata.
///
/// The [`Catalog`](crate::management::catalog::Catalog) only needs this metadata to list backups
/// and make decisions about the [`BackupStatus`].
///
/// # Difference between `Backup` and `Archive`
///
/// Indeed, each backup corresponds to an archive file (see
/// [`Archive`](crate::management::archive::v1::Archive)), but `Backup` provides only information
/// which can be derived from the file name. On the other side, an `Archive` requires to open the
/// file, deal with the format, parse the metadata, etc.
pub struct Backup {
    /// Path to the backup file.
    pub filepath: PathBuf,

    /// Backup date.
    pub creation_date: NaiveDateTime,
}

/// Which subset of backups to print.
#[derive(Debug, Clone, ValueEnum)]
pub enum BackupStatus {
    /// Retainable backups only.
    Retainable,
    /// Disposable backups only.
    Disposable,
}

/// Details such as the number of sessions, windows and panes in a backup.
///
/// These counts are displayed after the commands such as `save`, `restore`, or
/// `catalog list --details`.
#[derive(Debug)]
pub struct BackupDetails {
    /// Format version of the backup.
    pub version: String,

    /// Number of sessions in a backup.
    pub num_sessions: u16,

    /// Number of windows in a backup.
    pub num_windows: u16,

    /// Number of panes in a backup.
    pub num_panes: u16,
}

impl fmt::Display for BackupDetails {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!(
            "{} sessions ({} windows, {} panes)",
            self.num_sessions, self.num_windows, self.num_panes,
        ))
    }
}
