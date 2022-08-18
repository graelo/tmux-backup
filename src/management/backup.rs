//! High-level representation of a backup for catalog operations and reporting.

use std::fmt;
use std::path::PathBuf;

use chrono::NaiveDateTime;
use clap::ValueEnum;

/// Quick access, high-level representation of a backup.
///
/// `Backup` provides only information which can be derived from the file name, avoiding to open
/// the file, deal with the format, parse the metadata, etc.
///
/// This is sufficient for the [`Catalog`](crate::management::catalog::Catalog) to list backups
/// and decide whether or not a backup should be deleted or kept.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
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

    /// Purgeable backups only.
    Purgeable,
}

impl fmt::Display for BackupStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BackupStatus::Retainable => write!(f, "{:12}", "retainable"),
            BackupStatus::Purgeable => write!(f, "{:12}", "purgeable"),
        }
    }
}
